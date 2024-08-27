use std::env;
use std::io::{self, Write};
use std::process::{Command, exit};
use std::path::Path;
const MAX_INPUT_LENGTH: usize = 1000;
const MAX_ARGS_LENGTH: usize = 100;
// TODO: Modulize the code
fn main() {
    let args: Vec<String> = env::args().collect();
    let display_cwd = args.len() > 1;

    loop {
        if display_cwd {
            let cwd = env::current_dir().unwrap_or_else(|_| Path::new("").to_path_buf()); // Maybe permission denied ?
            let cwd_str = cwd.to_str().unwrap_or("");
            eprint!("{}$ ", cwd_str);
        } else {
            eprint!("$ ");
        }
        io::stderr().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            // again rare case, but if it happens, we should exit the shell 
            // e.g. Interrupted System Call ?
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input.len() > MAX_INPUT_LENGTH {
            eprintln!("error: command line too long");
            continue;
        }
        let args = match parse_command(input) {
            Ok(args) => {
                if args.len() > MAX_ARGS_LENGTH {
                    eprintln!("error: too many arguments");
                    continue;
                }
                args
            }
            Err(e) => {
                eprintln!("error: {}", e);
                continue;
            }
        };

        match args[0].as_str() {
            "cd" => {
                if args.len() != 2 {
                    eprintln!("error: cd requires exactly one argument");
                } else if let Err(e) = env::set_current_dir(&args[1]) {
                    eprintln!("error: cd failed: {}", e);
                }
                continue;
            }
            "exit" => {
                exit(0);
            }
            _ => {}
        }
        let status = if Path::new(&args[0]).is_absolute() {
            Command::new(&args[0]).args(&args[1..]).status()
        } else {
            let paths = env::var("PATH").unwrap_or_default()
                .split(':')
                .map(|path| Path::new(path).join(&args[0]))
                .find(|path| path.exists());
    
            match paths {
                Some(path) => Command::new(path).args(&args[1..]).status(),
                None => {
                    eprintln!("error: command not found");
                    continue;
                }
            }
        };

        match status {
            Ok(status) => {
                if !status.success() {
                    eprintln!("error: command exited with code {}", status.code().unwrap_or(-1));
                }
            }
            Err(e) => {
                eprintln!("error: failed to execute command: {}", e);
            }
        }
    }
}

fn parse_command(input: &str) -> Result<Vec<String>, &'static str> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    //TODO: handle '$' to expand variables
    for c in input.chars() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_single_quote || in_double_quote {
        return Err("mismatched quotes");
    }

    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}