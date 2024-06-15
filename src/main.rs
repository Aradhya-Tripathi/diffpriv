/*
Configurations will later be stored permanently once the interface for interaction is decided.
For now, all fields need to be filled on boot.
We will also be analyzing the query before running it to disallow unwanted query runs.
Currently, we have access to everything, but strict query checking will be implemented later.
*/
use diffpriv::database::database::Database;
use diffpriv::database::schema::{Column, Schema};
use diffpriv::query::analyzer;
use diffpriv::transforms::laplace_transform;

use std::collections::HashMap;
use std::io::{self, Write};

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
    let aggregate_functions: Vec<&str> = vec!["sum(", "avg("];
    let mut index = 0;

    if requested.contains(&"*".to_string()) {
        // Wildcard should mean we are queries everything.
        return existing;
    }

    while index < existing.len() {
        for func in aggregate_functions.iter() {
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
    // Test function!
    // If usage does not exist that means this query is not trying to get the
    // average of a perticular row, instead it's query the whole row or something else
    // which in any case is not allowed!
    // let table_in_question = used_tables
    //     .iter()
    //     .find(|table| table.columns == used_columns)
    //     .unwrap();
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
                    laplace_transform(
                        true_value,
                        column.sensitivity,
                        privacy_budget_map
                            .get::<String>(&column.table_name)
                            .unwrap()
                            .clone(),
                    )
                })
            })
        })
        .collect()
}

fn main() {
    println!("------CONFIGURATION-------");
    print!("Database Path/URI> ");
    io::stdout().flush().unwrap();

    let mut database_uri = String::new();
    io::stdin().read_line(&mut database_uri).unwrap();
    let mut database_connection = Database::new(&database_uri).unwrap();
    println!("{database_connection}");

    println!("Generating database schema...");
    let mut database_tables = Schema::from_connection(&mut database_connection);

    let mut privacy_budget_map = HashMap::new();
    // Setting sensitivity for each column in all the tables and the table privacy budget.
    for table in database_tables.iter_mut() {
        println!("Setting configurations for {}", table.name);
        table.columns.iter_mut().for_each(|column| {
            let mut sensitivity = String::new();
            print!("Enter sensitivity for {}: ", column.name);
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut sensitivity).unwrap();
            let sensitivity = sensitivity.trim().parse::<f64>();
            match sensitivity {
                Ok(value) => column.sensitivity = value,
                Err(_) => eprintln!("Error parsing sensitivity falling back to default 0.0"),
            }
        });
        let mut privacy_budget = String::new();
        print!("Enter privacy budget for {}: ", table.name);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut privacy_budget).unwrap();
        let table_privacy = match privacy_budget.trim().parse::<f64>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("Error parsing privacy budget falling back to default 0.0");
                0.0
            }
        };
        privacy_budget_map.insert(table.name.clone(), table_privacy);
    }

    println!("-------END CONFIGURATION-------");

    loop {
        let mut query = String::new();
        print!("query> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut query).unwrap();

        let analyzer = analyzer::SqlAnalyzer::new(&query);
        let requested_columns = analyzer.columns_from_sql();
        // let requested_tables = analyzer.tables_from_sql();
        let existing_columns = database_tables
            .iter()
            .flat_map(|table| table.columns.clone())
            .collect();

        let used_columns = get_used_columns(requested_columns, existing_columns);
        // Sensitivity issue now new columns have a sensitivity setting however old table columns don't!

        let query_result = database_connection.execute_query(&query);
        let transformed_query_results =
            apply_transforms(used_columns, query_result, &privacy_budget_map);

        println!("{:?}", transformed_query_results);
    }
}
