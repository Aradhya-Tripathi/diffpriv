#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
/*
We will also be analyzing the query before running it to disallow unwanted query runs.
Currently, we have access to everything, but strict query checking will be implemented later.
We also don't allow * queries since we need to manage the privacy & sensitivity of each column
therefore something like: select count(*) from XYX; is treated as an illegal query.
Note - The password for the database server is generating on the fly.
*/
use diffpriv::database::database::Database;
use diffpriv::database::schema::{Column, Schema, Table};
use diffpriv::query::analyzer;
use diffpriv::transforms::laplace_transform;

use std::collections::HashMap;
use std::io::{self, BufReader, Write};
use std::{fs, process};

use serde_json::{self, Value};

static mut ALLOWED_AGGREGATIONS: Vec<String> = vec![];

/// Identifies and returns the columns used in the requested query.
/// Essentially `Column` construction from the requested columns detected
///
/// # Arguments
///
/// * `requested` - A vector of strings representing the requested columns or functions.
/// * `existing` - A vector of existing columns in the database.
///
/// # Returns
///
/// A vector of columns that are used in the query.
fn _get_used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
    let mut used_columns: Vec<Column> = vec![];
    let mut index = 0;

    while index < existing.len() {
        // mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
        unsafe {
            for func in ALLOWED_AGGREGATIONS.iter() {
                if requested.contains(&format!("{func}{})", existing[index].name)) {
                    existing[index].usage = Some(format!("{func}{})", existing[index].name));
                    used_columns.push(existing[index].to_owned());
                }
            }
        }

        if requested.contains(&existing[index].name) {
            used_columns.push(existing[index].to_owned());
        }

        index += 1;
    }
    used_columns
}
/// Applies Laplace transforms to the query results based on column sensitivity.
///
/// # Arguments
///
/// * `used_columns` - A vector of columns that are used in the query.
/// * `query_result` - A vector of hashmaps representing the query result rows.
///
/// # Returns
///
/// A vector of f64 representing the transformed query results.
fn _apply_transforms(
    used_columns: Vec<Column>,
    query_result: Vec<HashMap<String, String>>,
    privacy_budget_map: &HashMap<String, f64>,
) -> Vec<HashMap<String, f64>> {
    // If usage does not exist that means this query is not trying to get the
    // average of a perticular row, instead it's query the whole row or something else
    // which in any case is not allowed!
    let usage_to_column: HashMap<&String, &Column> = used_columns
        .iter()
        .filter_map(|column| column.usage.as_ref().map(|usage| (usage, column)))
        .collect();

    query_result
        .iter()
        .flat_map(|result| {
            result.iter().filter_map(|(k, v)| {
                let mut result_map: HashMap<String, f64> = HashMap::new();
                usage_to_column.get(&k).map(|&column| {
                    let true_value = v.parse::<f64>().unwrap(); // We won't be entering this block if the query is not an aggregate query
                    let table_budget = privacy_budget_map
                        .get::<String>(&column.table_name)
                        .unwrap()
                        .clone();
                    if table_budget <= 0.0 {
                        println!(
                            "Ran out of budget for {} expect invalid query results!",
                            &column.table_name
                        )
                    }
                    result_map.insert(
                        column.usage.as_ref().unwrap().to_owned(),
                        // There is no way that this unwrap fails since we are in
                        // usage_to_column already which means that column.usage exists!
                        laplace_transform(true_value, column.sensitivity, table_budget).to_owned(),
                    );
                    result_map
                })
            })
        })
        .collect()
}

fn configure_from_file(
    path_to_configuration: &str,
    database_type: &str,
) -> Result<(Vec<Table>, Database, HashMap<String, f64>), String> {
    // Todo handle malformed files.
    let file = match fs::File::open(path_to_configuration) {
        Ok(file) => file,
        Err(_) => {
            return Err("Failed to open configuration file!".to_string());
        }
    };

    let reader = BufReader::new(file);
    let configurations: Value = serde_json::from_reader(reader).unwrap();

    unsafe {
        ALLOWED_AGGREGATIONS = configurations
            .get("allowed_aggregations")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|val| val.as_str().unwrap().to_owned())
            .collect::<Vec<String>>();
    }

    match configurations.get(database_type.trim()) {
        Some(content) => {
            let database_uri: String;

            if content.get("in_memory").unwrap() == false {
                database_uri = format!(
                    "{}{}",
                    content.get("uri").unwrap().as_str().unwrap(),
                    content.get("database").unwrap().as_str().unwrap()
                );
            } else {
                database_uri = content
                    .get("database")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
            }

            let mut database_connection = match Database::new(&database_uri) {
                Ok(connection) => connection,
                Err(msg) => return Err(msg),
            };
            let mut database_tables = Schema::from_connection(&mut database_connection);
            let mut privacy_budget_map: HashMap<String, f64> = HashMap::new();

            for table in database_tables.iter_mut() {
                let table_settings = content.get("tables").unwrap().get(&table.name).unwrap();
                table.columns.iter_mut().for_each(|column| {
                    let sensitivity = table_settings
                        .get(&column.name)
                        .map(|val| val.as_f64().unwrap())
                        .unwrap_or_else(|| f64::INFINITY);
                    if sensitivity != f64::INFINITY {
                        println!(
                            "Setting sensitivity for {} to {}",
                            &column.name, &sensitivity
                        );
                    } else {
                        println!(
                            "Private column found setting sensitivity to {}",
                            &sensitivity
                        )
                    }
                    column.sensitivity = sensitivity;
                });

                let table_privacy = table_settings
                    .get("__table__privacy")
                    .unwrap()
                    .as_f64()
                    .unwrap();
                table.privacy_budget = table_privacy;
                privacy_budget_map.insert(table.name.clone(), table_privacy);
                println!(
                    "Setting table privacy for {} to {}",
                    &table.name, &table_privacy
                );
            }
            Ok((database_tables, database_connection, privacy_budget_map))
        }
        None => Err(format!("Invalid Database: {}", &database_type.trim())),
    }
}

#[tauri::command]
fn configure(config_path: String, database_type: String) -> Result<(), String> {
    let configured = configure_from_file(&config_path, &database_type);
    match configured {
        Ok((database_tables, database_connection, privacy_budget_map)) => {
            println!("We are configured!");
            Ok(())
        }
        Err(msg) => Err(msg),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![configure])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
