#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
/*
We will also be analyzing the query before running it to disallow unwanted query runs.
Currently, we have access to everything, but strict query checking will be implemented later.
We also don't allow * queries since we need to manage the privacy & sensitivity of each column
therefore something like: select count(*) from XYX; is treated as an illegal query.
Note - The password for the database server is generating on the fly.
*/
use diffpriv::database::schema::Table;
use diffpriv::database::{database::Database, schema::Schema};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

struct AppState {
    pub connection: Mutex<Option<Database>>,
    pub schema: Mutex<Option<Vec<Table>>>,
}

#[tauri::command]
fn set_sensitivities(
    app_state: State<'_, Arc<AppState>>,
    sensitivities: HashMap<String, HashMap<String, f64>>,
) {
    let database_tables = app_state.schema.lock().unwrap().clone().unwrap();
    for mut table in database_tables {
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
    }
}

#[tauri::command]
fn get_tables(app_state: State<'_, Arc<AppState>>) -> Result<Vec<Table>, String> {
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
            set_sensitivities
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
