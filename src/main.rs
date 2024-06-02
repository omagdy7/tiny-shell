#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    process::exit,
};

struct ShellCommand<'a> {
    cmd: &'a str,
    args: &'a [&'a str],
}

#[derive(Debug)]
struct Context {
    executbles: HashMap<String, PathBuf>,
}

impl<'a> From<&'a [&'a str]> for ShellCommand<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        ShellCommand {
            cmd: &value[0],
            args: &value[1..],
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

fn eval(command: &str, ctx: &Context) {
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
