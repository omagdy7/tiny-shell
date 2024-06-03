#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{exit, Command},
};

enum ShellCommandType {
    Builtin,
    Executable,
}

struct ShellCommand<'a> {
    cmd: &'a str,
    args: &'a [&'a str],
    command_type: ShellCommandType,
}

#[derive(Debug)]
struct Context {
    executbles: HashMap<String, PathBuf>,
}

impl<'a> From<&'a [&'a str]> for ShellCommand<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        let cmd = &value[0];
        let mut command_type = ShellCommandType::Executable;
        if BUILTINS.contains(&cmd) {
            command_type = ShellCommandType::Builtin;
        }
        ShellCommand {
            cmd,
            args: &value[1..],
            command_type,
        }
    }
}

const BUILTINS: [&str; 3] = ["echo", "exit", "type"];

fn get_executables(paths: &[&str], ctx: &mut Context) -> io::Result<()> {
    for path in paths {
        let entries = fs::read_dir(path);

        match entries {
            Ok(entrs) => {
                for entry in entrs {
                    let entry = entry?;
                    let path = entry.path();
                    let file_name = entry.file_name().into_string().unwrap();
                    ctx.executbles.insert(file_name, path);
                }
            }
            Err(_) => {}
        }
    }
    Ok(())
}

fn eval_builtin(command: &str, args: &[&str], ctx: &Context) {
    match command {
        "exit" => {
            let args = args[0].trim_end();
            let exit_num = args.parse::<i32>().expect("This should be parsable");
            exit(exit_num)
        }
        "echo" => {
            let line_to_print = args.join(" ");
            print!("{}", line_to_print);
        }
        "type" => {
            let cmd_to_check = args[0].trim_end();
            if BUILTINS.contains(&cmd_to_check) {
                println!("{} is a shell builtin", cmd_to_check);
            } else if ctx.executbles.contains_key(cmd_to_check) {
                println!(
                    "{} is {}",
                    cmd_to_check,
                    ctx.executbles[cmd_to_check].display()
                );
            } else {
                println!("{} not found", cmd_to_check);
            }
        }
        _ => {}
    }
}

fn eval_executable(command: &str, args: &[&str], ctx: &Context) {
    if ctx.executbles.contains_key(command) {
        let mut cmd = Command::new(command);
        let output = cmd.args(args).output().unwrap();
        // Check if the command was successful
        if output.status.success() {
            // Convert the output to a string and print it
            let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");
            println!("{}", stdout);
        } else {
            // If the command failed, print the error
            let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 sequence");
            eprintln!("Error:\n{}", stderr);
        }
    } else {
        println!("{} not found", command);
    }
}

fn eval(command: &str, ctx: &Context) {
    use ShellCommandType::*;
    let cmd_input = command
        .split(' ')
        .map(|cmd| cmd.trim())
        .collect::<Vec<&str>>();
    let shell_cmd = ShellCommand::from(cmd_input.as_slice());
    if command.split_once(' ').is_some() {
        let (cmd, args) = (shell_cmd.cmd, shell_cmd.args);
        match shell_cmd.command_type {
            Builtin => {
                eval_builtin(cmd, args, ctx);
            }
            Executable => {
                eval_executable(cmd, args, ctx);
            }
        }
    } else {
        println!("{}: command not found", command.trim_end());
    }
}

fn main() {
    // Wait for user input
    let mut ctx = Context {
        executbles: HashMap::new(),
    };
    let path = std::env::var("PATH").unwrap();
    let paths = path.split(':').collect::<Vec<&str>>();
    let _ = get_executables(&paths, &mut ctx);
    // dbg!(ctx);
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        stdin.read_line(&mut command).unwrap();
        eval(&command, &ctx);
    }
}
