#[allow(unused_imports)]
use std::io::{self, Write};

fn eval(command: &str) {
    println!("{}: command not found", command.trim_end());
}

fn main() {
    // Uncomment this block to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    eval(&input);
}
