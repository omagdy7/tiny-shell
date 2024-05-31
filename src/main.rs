#[allow(unused_imports)]
use std::io::{self, Write};

fn eval(command: &str) {
    println!("{}: command not found", command.trim_end());
}

fn main() {
    // Wait for user input
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        stdin.read_line(&mut command).unwrap();
        eval(&command);
    }
}
