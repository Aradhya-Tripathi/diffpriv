/*This is a read only sql analyzer!*/
use diffpriv::database::Database;
// use diffpriv::query::analyzer::SqlAnalyzer;

use std::io;

fn main() {
    let mut client_string: String = String::new();
    io::stdin()
        .read_line(&mut client_string)
        .expect("Failed to read Input");

    match Database::new(&client_string, "mysql") {
        Ok(content) => {
            println!("Connection successful!: {content:?}");
        }
        Err(msg) => {
            println!("ERROR: {msg}");
        }
    }
}
