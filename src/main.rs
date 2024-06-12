/*This is a read only sql analyzer!*/
use diffpriv::database::connect::Database;
use diffpriv::database::schema::Schema;
// use diffpriv::transforms::noise::transform;

use std::io;

fn main() {
    let mut client_string: String = String::new();
    io::stdin()
        .read_line(&mut client_string)
        .expect("Failed to read Input");

    match Database::new(&client_string, "sqlite") {
        Ok(connection) => {
            println!("Connection successful to {:?}", connection.flavour);
            Schema::from_connection(connection.connection)
        }
        Err(msg) => {
            println!("ERROR: {msg}");
        }
    }
    // transform();
}
// mysql://root:MARIADBPASSWORD@127.0.0.1:3306/test
