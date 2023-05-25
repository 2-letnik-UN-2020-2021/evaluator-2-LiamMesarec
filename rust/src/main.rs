use rust::tokenizer;
use rust::parser;
use rust::eval;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

fn main() {
    let mut variables = HashMap::new();
    variables.insert(String::from("x"), 1);
    variables.insert(String::from("y"), 3);

    for arg in std::env::args().into_iter().skip(1) {
        let mut reader = BufReader::new(File::open(&arg).expect("Error opening file."));

        match tokenizer::tokenize(&mut reader) {
            Err(error) => println!("\n{} in file {}", error, arg),
            Ok(tokens) => match parser::parse(&tokens) {
                Err(error) => println!("\n{} in file {}", error, arg),
                _ => match eval::parse(&tokens, &mut variables) {
                    Err(error) => println!("\n{} in file {}", error, arg),
                    Ok(_) => ()
                }
            }
        };
    }
}
