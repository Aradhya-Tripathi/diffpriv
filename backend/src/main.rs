#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
/*
We also don't allow * queries since we need to manage the privacy & sensitivity of each column
therefore something like: select count(*) from XYX; is treated as an illegal query.
Note - The password for the database server is generating on the fly.
*/
use diffpriv::database::database::Database;
use diffpriv::database::schema::{Column, Schema, Table};
use diffpriv::query::analyzer;
use diffpriv::transforms::laplace_transform;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

static ALLOWED_AGGREGATIONS: [&str; 5] = ["sum(", "avg(", "count(", "min(", "max("];

struct AppState {
    pub connection: Mutex<Option<Database>>,
    pub schema: Mutex<Option<Vec<Table>>>,
}

/// Applies differential privacy transformations to the query results.
///
/// # Parameters
/// - `used_columns`: A vector of columns used in the query.
/// - `query_result`: A vector of hashmaps representing the query results.
/// - `budget`: The privacy budget for the transformation.
///
/// # Returns
/// A vector of hashmaps with transformed (noised) query results.
/// _Documentation generated by ChatGPT._
fn apply_transforms(
    used_columns: Vec<Column>,
    query_result: Vec<HashMap<String, String>>,
    budget: f64,
) -> Vec<HashMap<String, f64>> {
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
                    // We need unwrap_or_default to handle Null and we are treating
                    // nulls as 0 (my decision)
                    let true_value = v.parse::<f64>().unwrap_or_default();
                    if budget <= 0.0 {
                        println!(
                            "Ran out of budget for {} expect invalid query results!",
                            &column.table_name
                        )
                    }
                    result_map.insert(
                        column.usage.as_ref().unwrap().to_owned(),
                        laplace_transform(true_value, column.sensitivity, budget).to_owned(),
                    );
                    result_map
                })
            })
        })
        .collect()
}

/// Determines which columns are used in the query.
///
/// # Parameters
/// - `requested`: A vector of requested table names.
/// - `existing`: A vector of existing tables in the database.
///
/// # Returns
/// A vector of tables that are used in the query.
fn get_used_tables(requested: Vec<String>, existing: &Vec<Table>) -> Vec<Table> {
    // TODO: Fix the case sensitive issues in this since the table in the requested column
    // Is always going to be lowercase (that's just how the analyzer works either fix that)
    existing
        .iter()
        .filter(|table| requested.contains(&table.name.to_ascii_lowercase()))
        .map(|table| table.to_owned())
        .collect()
}

/// Determines which columns are used in the query.
///
/// # Parameters
/// - `requested`: A vector of requested column names.
/// - `existing`: A vector of existing columns in the database.
///
/// # Returns
/// A vector of columns that are used in the query.
/// _Documentation generated by ChatGPT._
fn get_used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
    let mut used_columns: Vec<Column> = vec![];
    let mut index = 0;

    while index < existing.len() {
        for func in ALLOWED_AGGREGATIONS.iter() {
            if requested.contains(&format!("{func}{})", existing[index].name)) {
                existing[index].usage = Some(format!("{func}{})", existing[index].name));
                used_columns.push(existing[index].to_owned());
            }
        }

        if requested.contains(&existing[index].name) {
            used_columns.push(existing[index].to_owned());
        }

        index += 1;
    }
    used_columns
}

/// Resets the sensitivities of all columns in the database schema.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn reset_sensitivities(app_state: State<'_, Arc<AppState>>) {
    let mut schema = app_state.schema.lock().unwrap();
    let mut database = app_state.connection.lock().unwrap();
    if let Some(database) = database.as_mut() {
        *schema = Some(Schema::from_connection(database))
    }
    println!("Reset sensitivities!");
}

/// Resets the database connection.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn reset_connection(app_state: State<'_, Arc<AppState>>) {
    let mut schema = app_state.schema.lock().unwrap();
    let mut database = app_state.connection.lock().unwrap();
    *schema = None;
    *database = None;
}

fn sanitize_input(input: &str) -> String {
    input.replace("“", "\"").replace("”", "\"")
}

/// Executes an SQL query with differential privacy applied.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// - `query`: The SQL query to be executed.
/// - `budget`: The privacy budget for the query.
///
/// # Returns
/// A result containing either the transformed query results or an error message.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn execute_sql(
    app_state: State<'_, Arc<AppState>>,
    query: String,
    budget: f64,
) -> Result<Vec<HashMap<String, f64>>, String> {
    let sanitized_query = sanitize_input(query.as_str());
    let mut has_budget = true;
    let mut database = app_state.connection.lock().unwrap();
    let connection = database.as_mut().unwrap();
    let mut schema = app_state.schema.lock().unwrap();
    let database_tables = schema.as_mut().unwrap();
    let analyzer = analyzer::SqlAnalyzer::new(&query);
    let requested_columns = analyzer.columns_from_sql();
    let requested_tables = analyzer.tables_from_sql();
    let existing_columns: Vec<Column> = database_tables
        .iter()
        .flat_map(|table| table.columns.clone())
        .collect();
    let used_columns = get_used_columns(requested_columns, existing_columns);
    let used_tables = get_used_tables(requested_tables, database_tables);

    // Deduct budget from this table
    for table in database_tables.iter_mut() {
        used_tables.iter().for_each(|used_table| {
            if table.name == used_table.name {
                let message = format!(
                    "Reducing {} budget from {} to {}",
                    table.name,
                    table.privacy_budget,
                    table.privacy_budget - budget,
                );
                println!("{message}");
                table.privacy_budget -= budget;
            }
        });
    }

    // Check here if the tables that are being used have enough budget to execute this query
    used_tables.iter().for_each(|table| {
        if table.privacy_budget <= 0.0 {
            has_budget = false;
        }
    });
    if has_budget {
        let query_result = connection.execute_query(&sanitized_query)?;
        let transformed_query_results = apply_transforms(used_columns, query_result, budget);
        return Ok(transformed_query_results);
    }
    Err("Insufficient budget!".to_string())
}

/// Sets the allowed privacy budget for each column after which no more queries are processed for that column
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// - `budgets`: A hashmap of table names to table budgets.
#[tauri::command]
fn set_budgets(
    app_state: State<'_, Arc<AppState>>,
    budgets: HashMap<String, f64>,
) -> Result<String, String> {
    let mut schema = app_state.schema.lock().unwrap();
    if let Some(database_tables) = schema.as_mut() {
        database_tables.iter_mut().for_each(|table| {
            let budget = budgets.get(&table.name).unwrap_or(&0.0).to_owned();
            table.privacy_budget = budget;
        });
        return Ok("Set table budget!".to_string());
    }
    Err("Unable to establish connection with the database!".to_string())
}

/// Sets the sensitivities for columns in the database schema.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// - `sensitivities`: A hashmap of table names to column sensitivities.
///
/// # Returns
/// A result indicating success or failure.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn set_sensitivities(
    app_state: State<'_, Arc<AppState>>,
    sensitivities: HashMap<String, HashMap<String, f64>>,
) -> Result<String, String> {
    let mut schema = app_state.schema.lock().unwrap();
    if let Some(database_tables) = schema.as_mut() {
        database_tables.iter_mut().for_each(|table| {
            let table_sensitivity = sensitivities.get(&table.name).unwrap(); // We will for sure have this in the sensitivities
            table.columns.iter_mut().for_each(|column| {
                let field_sensitivity = table_sensitivity.get(&column.name).unwrap();
                println!(
                    "Setting sensitivity for {} to {}",
                    &column.name, field_sensitivity
                );
                column.sensitivity = field_sensitivity.to_owned();
            })
        });
        Ok("Set sensitivities".to_string())
    } else {
        Err("Unable to set sensitivities".to_string())
    }
}

/// Retrieves the tables from the database schema.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
///
/// # Returns
/// A result containing either the list of tables or an error message.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn get_tables(app_state: State<'_, Arc<AppState>>) -> Result<Vec<Table>, String> {
    let tables = app_state.schema.lock().unwrap().clone();
    if tables.is_some() {
        Ok(tables.unwrap())
    } else {
        Err("No tables loaded yet!".to_string())
    }
}

/// Connects to the database at the specified path.
///
/// # Parameters
/// - `app_state`: The shared application state containing the database connection and schema.
/// - `database_path`: The path to the database file.
///
/// # Returns
/// A result indicating success or failure.
/// _Documentation generated by ChatGPT._
#[tauri::command]
fn connect(
    app_state: State<'_, Arc<AppState>>, // Arc since we share between multiple threads (Safely).
    database_path: String,
) -> Result<String, String> {
    let mut connection_gaurd = app_state.connection.lock().unwrap();
    let mut schema_gaurd = app_state.schema.lock().unwrap();
    if connection_gaurd.is_none() {
        match Database::new(&database_path) {
            Ok(mut connection) => {
                *schema_gaurd = Some(Schema::from_connection(&mut connection));
                *connection_gaurd = Some(connection);
                Ok("Connected".to_string())
            }
            Err(msg) => Err(msg),
        }
    } else {
        Ok("Already connected to the database!".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(AppState {
            connection: Mutex::new(None),
            schema: Mutex::new(None),
        }))
        .invoke_handler(tauri::generate_handler![
            connect,
            get_tables,
            set_sensitivities,
            execute_sql,
            reset_sensitivities,
            reset_connection,
            set_budgets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
