/*This is a read only sql analyzer!*/
use diffpriv::database::Database;

use std::io;

fn main() {
    let mut client_string: String = String::new();
    io::stdin()
        .read_line(&mut client_string)
        .expect("Failed to read Input");

    match Database::new(&client_string, "sqlite") {
        Ok(connection) => {
            println!("Connection successful to {:?}", connection.flavour);
        }
        Err(msg) => {
            println!("ERROR: {msg}");
        }
    }
}
