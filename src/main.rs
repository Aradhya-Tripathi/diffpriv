/*This is a read only sql analyzer!*/
use diffpriv::query::analyzer::SqlAnalyzer;
use std::io;

fn main() {
    let mut client_string: String = String::new();
    io::stdin()
        .read_line(&mut client_string)
        .expect("Failed to read Input");

    let analyzer = SqlAnalyzer::new(&client_string);
    println!("Columns: {:?}", analyzer.columns_from_sql());
    println!("Tables: {:?}", analyzer.tables_from_sql());
}
