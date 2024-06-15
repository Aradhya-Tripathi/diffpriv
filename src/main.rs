/*
All these configurations will later be stored somewhere
Once I decide on the interface I want to use to interact with this
for now on boot all fields need to filled.
We are also analyzing the query before running it to disallow
unwanted query runs (which will be implemented later right now we have access to everything)
*/

use diffpriv::database::database::Database;
use diffpriv::database::schema::{Column, Schema};
use diffpriv::query::analyzer;
use diffpriv::transforms::transform;

use std::collections::HashMap;
use std::io::{self, Write};

fn used_columns(requested: Vec<String>, mut existing: Vec<Column>) -> Vec<Column> {
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

fn apply_transforms(
    used_columns: Vec<Column>,
    query_result: Vec<HashMap<String, String>>,
) -> Vec<f64> {
    // Test function!
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
                    transform(true_value, column.sensitivity)
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
    }

    println!("-------END CONFIGURATION-------");

    loop {
        let mut query = String::new();
        print!("query> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut query).unwrap();

        let analyzer = analyzer::SqlAnalyzer::new(&query);
        let used_columns = used_columns(
            analyzer.columns_from_sql(),
            database_tables
                .iter()
                .flat_map(|table| table.columns.clone())
                .collect(),
        );
        let query_result = database_connection.execute_query(&query);
        println!("{:?}", apply_transforms(used_columns, query_result));
    }
}
