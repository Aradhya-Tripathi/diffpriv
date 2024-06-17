/*
We will also be analyzing the query before running it to disallow unwanted query runs.
Currently, we have access to everything, but strict query checking will be implemented later.
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
fn get_used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
    let mut used_columns: Vec<Column> = vec![];
    // let aggregate_functions: Vec<&str> = vec!["sum(", "avg("];
    let mut index = 0;

    if requested.contains(&"*".to_string()) {
        // Wildcard should mean we are queries everything.
        return existing;
    }

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
fn apply_transforms(
    used_columns: Vec<Column>,
    query_result: Vec<HashMap<String, String>>,
    privacy_budget_map: &HashMap<String, f64>,
) -> Vec<f64> {
    // If usage does not exist that means this query is not trying to get the
    // average of a perticular row, instead it's query the whole row or something else
    // which in any case is not allowed!
    let usage_to_column: HashMap<&String, &Column> = used_columns
        .iter()
        .filter_map(|column| column.usage.as_ref().map(|usage| (usage, column)))
        .collect();

    query_result
        .into_iter()
        .flat_map(|result| {
            result.into_iter().filter_map(|(k, v)| {
                usage_to_column.get(&k).map(|&column| {
                    let true_value = v
                        .parse::<f64>()
                        .expect("Illegal usage no aggregate used on this column!");
                    // This will later be removed and we will have
                    // A strict query checker before the query is actually executed!
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
                    laplace_transform(true_value, column.sensitivity, table_budget)
                })
            })
        })
        .collect()
}

fn configure_from_file(
    path_to_configuration: &str,
) -> (Vec<Table>, Database, HashMap<String, f64>) {
    // Todo handle malformed files.
    let mut database_type = String::new();

    let file = fs::File::open(path_to_configuration).expect("No configuration file found!");
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

    print!("Database Type> ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut database_type).unwrap();

    match configurations.get(&database_type.trim()) {
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

            let mut database_connection = Database::new(&database_uri).unwrap();
            let mut database_tables = Schema::from_connection(&mut database_connection);
            let mut privacy_budget_map: HashMap<String, f64> = HashMap::new();

            for table in database_tables.iter_mut() {
                let table_settings = content.get("tables").unwrap().get(&table.name).unwrap();

                table.columns.iter_mut().for_each(|column| {
                    let sensitivity = table_settings.get(&column.name).unwrap().as_f64().unwrap();
                    column.sensitivity = sensitivity;
                    println!(
                        "Setting sensitivity for {} to {}",
                        &column.name, &sensitivity
                    );
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

            (database_tables, database_connection, privacy_budget_map)
        }
        None => {
            eprintln!(
                "No such database {} found in configurations",
                &database_type.trim()
            );
            process::exit(-1);
        }
    }
}

fn main() {
    let mut configuration_path = String::new();
    print!("Configuration path> ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut configuration_path).unwrap();

    let (database_tables, mut database_connection, privacy_budget_map) =
        configure_from_file(&configuration_path.trim());

    loop {
        let mut query = String::new();
        print!("query> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut query).unwrap();

        let analyzer = analyzer::SqlAnalyzer::new(&query);
        let requested_columns = analyzer.columns_from_sql();
        let existing_columns = database_tables
            .iter()
            .flat_map(|table| table.columns.clone())
            .collect();

        let used_columns = get_used_columns(requested_columns, existing_columns);
        let query_result = database_connection.execute_query(&query);
        let transformed_query_results =
            apply_transforms(used_columns, query_result, &privacy_budget_map);

        println!("{:?}", transformed_query_results);
    }
}
