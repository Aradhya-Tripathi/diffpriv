/*This is a read only sql parser!*/
use diffpriv::lexer::Lexer;
use std::io;

fn main() {
    let mut client_string: String = String::new();
    io::stdin()
        .read_line(&mut client_string)
        .expect("Failed to read Input");

    let mut lexer = Lexer::new(client_string.to_owned());
    let tokens = lexer.parse();
    println!("{tokens:?}");
}
