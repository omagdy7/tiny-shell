#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn eval(command: &str) {
    if command.split_once(' ').is_some() {
        let (cmd, args) = command.split_once(' ').unwrap();
        let args = args.trim_end();
        let exit_num = args.parse::<i32>().expect("This should be parsable");
        match cmd {
            "exit" => exit(exit_num),
            _ => {}
        }
    } else {
        println!("{}: command not found", command.trim_end());
    }
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
