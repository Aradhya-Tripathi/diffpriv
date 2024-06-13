use diffpriv::database::connect::Database;
use diffpriv::database::schema::Schema;
use diffpriv::query::analyzer;
use std::io::{self, Write};

pub fn main() {
    print!("Database Path/URI> ");
    io::stdout().flush().unwrap();
    let mut database_uri = String::new();
    io::stdin().read_line(&mut database_uri).unwrap();
    let mut database_connection = Database::new(&database_uri).unwrap();

    println!("Generating database schema...");

    let _database_tables = Schema::from_connection(&mut database_connection);

    loop {
        let mut query = String::new();
        print!("query> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut query).unwrap();
        let analyzer = analyzer::SqlAnalyzer::new(&query);

        println!(
            "tables: {:?} columns: {:?}",
            analyzer.tables_from_sql(),
            analyzer.columns_from_sql()
        );

        database_connection.execute_query(&query);
    }
}
