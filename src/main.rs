#![forbid(unsafe_code)] // Disallow all unsafe code
#![warn(unused_results)] // Warn on unused Result or Option types
#![warn(clippy::all)] // Enable all Clippy lints
#![warn(clippy::pedantic)] // Enable strict Clippy lints
#![warn(clippy::unwrap_used)] // Warn when unwrap or expect are used
#![warn(clippy::panic)] // Warn on use of panic!
#![warn(clippy::result_unwrap_used)] // Warn on Result.unwrap()
#![warn(clippy::option_unwrap_used)] // Warn on Option.unwrap()
#![warn(clippy::redundant_clone)] // Warn on unnecessary clones
#![warn(clippy::needless_pass_by_value)] // Suggest borrowing instead of copying

use anyhow::anyhow;
use anyhow::Result;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::{exit, Command},
};

#[derive(Debug)]
enum ShellCommandType {
    Builtin,
    Executable,
}

#[derive(Debug)]
struct ShellCommand<'a> {
    cmd: &'a str,
    args: Vec<&'a str>,
    command_type: ShellCommandType,
}

impl<'a> From<&'a [&'a str]> for ShellCommand<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        let cmd = &value[0];
        let mut command_type = ShellCommandType::Executable;
        if BUILTINS.contains(&cmd) {
            command_type = ShellCommandType::Builtin;
        }
        let mut args = vec![];
        for arg in value.iter().skip(1) {
            if arg.starts_with("'") && arg.ends_with("'") {
                args.push(&arg[1..arg.len() - 2]);
            } else {
                args.push(arg);
            }
        }
        ShellCommand {
            cmd,
            args,
            command_type,
        }
    }
}

#[derive(Debug, Clone)]
struct Context {
    executables: HashMap<String, PathBuf>,
    current_working_dir: PathBuf,
}

fn change_directory(ctx: &mut Context, args: &[&str]) -> Result<()> {
    match args.len() {
        0 => {
            // Go to home directory when no arguments are provided
            ctx.current_working_dir = PathBuf::from(HOME);
            std::env::set_current_dir(&ctx.current_working_dir)?;
        }
        1 => {
            let path = args[0];
            let new_path = resolve_path(ctx, path)?;

            // Attempt to change directory and update context
            if new_path.exists() && new_path.is_dir() {
                ctx.current_working_dir = new_path.clone();
                std::env::set_current_dir(&new_path)?;
            } else {
                return Err(anyhow!("cd: {}: No such directory", new_path.display()));
            }
        }
        _ => {
            println!("cd: please provide only one directory");
        }
    }

    Ok(())
}

fn resolve_path(ctx: &Context, path: &str) -> Result<PathBuf> {
    let current_dir = &ctx.current_working_dir;

    // Handle various path scenarios
    let resolved_path = match path {
        // Home directory variations
        "" | "~" | "~/" => PathBuf::from(HOME),

        // Home directory with subpath
        path if path.starts_with("~/") => PathBuf::from(HOME).join(&path[2..]),

        // Parent directory
        ".." => current_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| current_dir.clone()),

        // Relative paths starting with ../
        path if path.starts_with("../") => {
            let mut path_buf = current_dir.clone();
            let remaining_path = &path[3..];

            // Remove parent directories as needed
            let _ = path_buf.pop();
            path_buf.join(remaining_path)
        }

        // Relative paths starting with ./
        path if path.starts_with("./") => current_dir.join(&path[2..]),

        // Absolute paths
        path if path.starts_with('/') => PathBuf::from(path),

        // Relative paths
        _ => current_dir.join(path),
    };

    match resolved_path.canonicalize() {
        Ok(resolved) => Ok(resolved),
        Err(_) => {
            return Err(anyhow!("cd: {}: No such file or directory", path));
        }
    }
}

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
const HOME: &'static str = env!("HOME");

fn populate_executables(paths: &[&str], ctx: &mut Context) -> Result<()> {
    for path in paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(file_name) = entry.file_name().to_str() {
                    let _ = ctx.executables.insert(file_name.to_string(), path);
                }
            }
        }
    }
    Ok(())
}

fn eval_builtin(command: &str, args: &[&str], ctx: &mut Context) -> Result<()> {
    match command {
        "exit" => {
            if args.len() == 0 {
                exit(0)
            }
            let args = args[0].trim_end();
            let exit_num = args.parse::<i32>()?;
            exit(exit_num)
        }
        "echo" => {
            let redirection_pos = args.iter().position(|&x| x == "1>" || x == ">");
            let mut cmd_args = args;
            let mut redirection_path = String::from("");
            if redirection_pos.is_some() {
                cmd_args = &args[0..redirection_pos.unwrap()];
                redirection_path = args.last().unwrap().to_string();
            }

            if redirection_pos.is_some() {
                let mut file_content = cmd_args.join(" ");
                file_content.push('\n');
                let mut file = File::create(redirection_path)?;
                file.write_all(file_content.as_bytes())?
            } else {
                let line_to_print = cmd_args.join(" ").trim_end().to_owned();
                println!("{}", line_to_print);
            }
            Ok(())
        }
        "type" => {
            let cmd_to_check = args[0].trim_end();
            if BUILTINS.contains(&cmd_to_check) {
                println!("{} is a shell builtin", cmd_to_check);
                Ok(())
            } else if ctx.executables.contains_key(cmd_to_check) {
                println!(
                    "{} is {}",
                    cmd_to_check,
                    ctx.executables[cmd_to_check].display()
                );
                Ok(())
            } else {
                println!("{}: not found", cmd_to_check);
                Ok(())
            }
        }
        "pwd" => {
            let cwd = ctx
                .current_working_dir
                .to_str()
                .ok_or_else(|| anyhow!("Option was none whoops!"))?;
            if cwd.ends_with("/") {
                println!("{}", PathBuf::from(&cwd[0..cwd.len() - 1]).display())
            } else {
                println!("{}", ctx.current_working_dir.display())
            }
            Ok(())
        }
        "cd" => change_directory(ctx, args),
        _ => Ok(()),
    }
}

fn eval_executable(command: &str, args: &[&str], ctx: &Context) -> Result<()> {
    if ctx.executables.contains_key(command) {
        let full_path_cmd = ctx.executables[command].to_str().unwrap();
        let mut cmd = Command::new(full_path_cmd);
        let redirection_pos = args.iter().position(|&x| x == "1>" || x == ">");
        let mut cmd_args = args;
        let mut redirection_path = String::from("");
        if redirection_pos.is_some() {
            cmd_args = &args[0..redirection_pos.unwrap()];
            redirection_path = args.last().unwrap().to_string();
        }
        let output = cmd.args(cmd_args).output().unwrap();
        // Check if the command was successful
        if output.status.success() {
            if redirection_pos.is_some() {
                let mut file = File::create(redirection_path)?;
                file.write_all(&output.stdout)?;
            } else {
                // Convert the output to a string and print it
                let stdout = String::from_utf8(output.stdout)?;
                print!("{}", stdout);
            }
            Ok(())
        } else {
            if redirection_pos.is_some() {
                let mut file = File::create(redirection_path)?;
                file.write_all(&output.stdout)?;
            }
            // If the command failed, print the error
            let stderr = String::from_utf8(output.stderr)?;
            let (cmd, err) = stderr.split_once(':').unwrap();
            let cmd = Path::new(cmd).file_name().unwrap().to_str().unwrap();
            let err = err.trim_end();
            eprintln!("{}:{}", cmd, err);
            Ok(())
        }
    } else {
        // TODO: Instead of repoulating the executables at every command instead when a comand
        // isn't found try to repopulate the exectables and try again if it fails then the
        // exectable can't indeed be found
        println!("{}: command not found", command);
        Ok(())
    }
}

fn parse_command(command: &str) -> Vec<String> {
    let mut inside_single_quotes = false;
    let mut inside_double_quotes = false;
    let mut current = String::new();
    let mut result = Vec::new();
    let mut chars = command.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' if inside_double_quotes => {
                if let Some(&next_char) = chars.peek() {
                    if ['\\', '$', '"'].contains(&next_char) {
                        current.push(next_char);
                        let _ = chars.next();
                    } else {
                        current.push(c)
                    }
                }
            }
            '\\' if !inside_single_quotes => {
                if let Some(&next_char) = chars.peek() {
                    current.push(next_char);
                    let _ = chars.next();
                }
            }
            // '1' if !(inside_double_quotes && inside_single_quotes) => {
            //     if let Some(&next_char) = chars.peek() {
            //         if next_char == '>' {
            //             current.push('1');
            //             current.push('>');
            //             let _ = chars.next();
            //         }
            //     }
            // }
            // '>' if !(inside_double_quotes && inside_single_quotes) => {
            //     current.push('>');
            // }
            '\'' if !inside_double_quotes => inside_single_quotes = !inside_single_quotes,
            '"' if !inside_single_quotes => inside_double_quotes = !inside_double_quotes,
            ' ' if !inside_single_quotes && !inside_double_quotes => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            '\n' => {}
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn eval(command: &str, ctx: &mut Context) -> Result<()> {
    use ShellCommandType::*;
    let cmd_input = parse_command(command);
    let cmd_input = cmd_input.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let shell_cmd = ShellCommand::from(cmd_input.as_slice());
    let (cmd, args) = (shell_cmd.cmd, shell_cmd.args);
    match shell_cmd.command_type {
        Builtin => {
            eval_builtin(cmd, &args, ctx)?;
        }
        Executable => {
            eval_executable(cmd, &args, ctx)?;
        }
    }
    Ok(())
}

fn main() {
    let mut ctx = Context {
        executables: HashMap::new(),
        current_working_dir: env::current_dir().expect("Shouldn't fail?"),
    };

    let path = std::env::var("PATH").unwrap();
    let paths = path.split(':').collect::<Vec<&str>>();
    let _ = populate_executables(&paths, &mut ctx);
    let stdin = io::stdin();

    let mut is_first_command = true;

    loop {
        // print!(
        //     "[{}]$ ",
        //     ctx.current_working_dir
        //         .file_name()
        //         .unwrap()
        //         .to_str()
        //         .unwrap()
        // );
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        let _ = stdin.read_line(&mut command).unwrap();
        if is_first_command {
            let _ = populate_executables(&paths, &mut ctx);
            is_first_command = false;
        }
        if let Err(e) = eval(&command, &mut ctx) {
            eprintln!("{:?}", e);
        }
    }
}
