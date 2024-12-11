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
use std::io::{self, Write};
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
    args: &'a [&'a str],
    command_type: ShellCommandType,
}

#[derive(Debug)]
struct Context {
    executables: HashMap<String, PathBuf>,
    current_working_dir: PathBuf,
}

fn _listdir(p: &PathBuf) -> Result<Vec<PathBuf>, io::Error> {
    fs::read_dir(p)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
}

fn change_directory(ctx: &mut Context, args: &[&str]) -> Result<(), anyhow::Error> {
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

fn resolve_path(ctx: &Context, path: &str) -> Result<PathBuf, anyhow::Error> {
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

impl<'a> From<&'a [&'a str]> for ShellCommand<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        let cmd = &value[0];
        let mut command_type = ShellCommandType::Executable;
        if BUILTINS.contains(&cmd) {
            command_type = ShellCommandType::Builtin;
        }
        let args = if value.len() == 1 { &[] } else { &value[1..] };
        ShellCommand {
            cmd,
            args,
            command_type,
        }
    }
}

const BUILTINS: [&str; 5] = ["echo", "exit", "type", "pwd", "cd"];
const PATH: &'static str = env!("PATH");
const HOME: &'static str = env!("HOME");

fn populate_executables(paths: &[&str], ctx: &mut Context) -> Result<(), anyhow::Error> {
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

fn eval_builtin(command: &str, args: &[&str], ctx: &mut Context) -> Result<(), anyhow::Error> {
    match command {
        "exit" => {
            let args = args[0].trim_end();
            let exit_num = args.parse::<i32>()?;
            exit(exit_num)
        }
        "echo" => {
            let line_to_print = args.join(" ").trim_end().to_owned();
            println!("{}", line_to_print);
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

fn eval_executable(command: &str, args: &[&str], ctx: &Context) -> Result<(), anyhow::Error> {
    if ctx.executables.contains_key(command) {
        let full_path_cmd = ctx.executables[command].to_str().unwrap();
        let mut cmd = Command::new(full_path_cmd);
        let output = cmd.args(args).output().unwrap();
        // Check if the command was successful
        if output.status.success() {
            // Convert the output to a string and print it
            let stdout = String::from_utf8(output.stdout)?;
            print!("{}", stdout);
            Ok(())
        } else {
            // If the command failed, print the error
            let stderr = String::from_utf8(output.stderr)?;
            eprintln!("Error:\n{}", stderr);
            Ok(())
        }
    } else {
        println!("{}: command not found", command);
        Ok(())
    }
}

fn eval(command: &str, ctx: &mut Context) -> Result<(), anyhow::Error> {
    use ShellCommandType::*;
    let cmd_input = command
        .split(' ')
        .map(|cmd| cmd.trim())
        .collect::<Vec<&str>>();
    let shell_cmd = ShellCommand::from(cmd_input.as_slice());
    let (cmd, args) = (shell_cmd.cmd, shell_cmd.args);
    match shell_cmd.command_type {
        Builtin => {
            eval_builtin(cmd, args, ctx)?;
        }
        Executable => {
            eval_executable(cmd, args, ctx)?;
        }
    }
    Ok(())
}

fn main() {
    // Wait for user input
    let mut ctx = Context {
        executables: HashMap::new(),
        current_working_dir: env::current_dir().expect("Shouldn't fail?"),
    };
    let paths = PATH.split(':').collect::<Vec<&str>>();
    let _ = populate_executables(&paths, &mut ctx);
    // dbg!(&ctx.executbles);
    let stdin = io::stdin();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        let _ = stdin.read_line(&mut command).unwrap();
        if let Err(e) = eval(&command, &mut ctx) {
            eprintln!("{:?}", e);
        }
    }
}
