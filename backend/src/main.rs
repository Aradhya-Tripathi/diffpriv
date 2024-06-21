#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
/*
We will also be analyzing the query before running it to disallow unwanted query runs.
Currently, we have access to everything, but strict query checking will be implemented later.
We also don't allow * queries since we need to manage the privacy & sensitivity of each column
therefore something like: select count(*) from XYX; is treated as an illegal query.
Note - The password for the database server is generating on the fly.
*/
use diffpriv::database::schema::Table;
use diffpriv::database::{database::Database, schema::Column, schema::Schema};
use diffpriv::query::analyzer;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

static ALLOWED_AGGREGATIONS: [&str; 3] = ["sum(", "avg(", "count("];

struct AppState {
    pub connection: Mutex<Option<Database>>,
    pub schema: Mutex<Option<Vec<Table>>>,
}

fn get_used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
    let mut used_columns: Vec<Column> = vec![];
    let mut index = 0;

    while index < existing.len() {
        // mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior

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

#[tauri::command]
fn execute_sql(app_state: State<'_, Arc<AppState>>, query: String) {
    // MutexGaurd allows the automatic unlocking mechanism to work, we don't
    // Need to explicitly call unlock we can just make MutexGaurd go out of scope.
    // We know for a fact that at this point we will have a value in the connection.
    let mut database = app_state.connection.lock().unwrap();
    let connection = database.as_mut().unwrap();
    let schema = app_state.schema.lock().unwrap();
    let database_tables = schema.as_ref().unwrap();

    let analyzer = analyzer::SqlAnalyzer::new(&query);
    let requested_columns = analyzer.columns_from_sql();
    let existing_columns: Vec<Column> = database_tables
        .iter()
        .flat_map(|table| table.columns.clone())
        .collect();

    let used_columns = get_used_columns(requested_columns, existing_columns);
    let query_result = connection.execute_query(&query);

    println!("{used_columns:?}");
    println!("{query_result:?}");
}

#[tauri::command]
fn set_sensitivities(
    app_state: State<'_, Arc<AppState>>,
    sensitivities: HashMap<String, HashMap<String, f64>>,
) {
    let mut database_tables = app_state.schema.lock().unwrap().clone().unwrap();
    database_tables.iter_mut().for_each(|table| {
        let table_sensitivity = sensitivities.get(&table.name).unwrap(); // We will for sure have this in the sensitivities
        table.columns.iter_mut().for_each(|colum| {
            let field_sensitivity = table_sensitivity.get(&colum.name).unwrap();
            // We will for sure have sensitivity for this field.
            println!(
                "Setting sensitivity for {} to {}",
                &colum.name, field_sensitivity
            );
            colum.sensitivity = field_sensitivity.to_owned();
        })
    });
}

#[tauri::command]
fn get_tables(app_state: State<'_, Arc<AppState>>) -> Result<Vec<Table>, String> {
    // Todo apt. error returns (Don't send over simple strings)
    let tables = app_state.schema.lock().unwrap().clone();
    if tables.is_some() {
        Ok(tables.unwrap())
    } else {
        Err("No tables loaded yet!".to_string())
    }
}

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
        Err("Already connected to the database!".to_string())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
