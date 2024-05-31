#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

struct ShellCommand<'a> {
    cmd: &'a str,
    args: &'a [&'a str],
}

impl<'a> From<&'a [&'a str]> for ShellCommand<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        ShellCommand {
            cmd: &value[0],
            args: &value[1..],
        }
    }
}

fn eval(command: &str) {
    let v = command.split(' ').collect::<Vec<&str>>();
    let shell_cmd = ShellCommand::from(v.as_slice());
    if command.split_once(' ').is_some() {
        let (cmd, args) = (shell_cmd.cmd, shell_cmd.args);
        match cmd {
            "exit" => {
                let args = args[0].trim_end();
                let exit_num = args.parse::<i32>().expect("This should be parsable");
                exit(exit_num)
            }
            "echo" => {
                let line_to_print = args.join(" ");
                print!("{}", line_to_print);
            }
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
