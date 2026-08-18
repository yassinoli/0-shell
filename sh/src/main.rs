use sh::{execute, tokenize, Status, TokenizeState};
use std::io::{self, Write};
use std::process;

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();
    let mut buffer = String::new();
    let mut exit_code = 0;

    loop {
        if buffer.is_empty() {
            print!("$ ");
        } else {
            print!("> ");
        }
        let _ = io::stdout().flush();

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                if buffer.is_empty() && line.trim().is_empty() {
                    continue;
                }

                buffer.push_str(&line);
                
                    if buffer.ends_with("\\\n") {
                        buffer.pop();
                        buffer.pop();

                        continue;
                    }

                match tokenize(&buffer) {
                    TokenizeState::Incomplete => continue,
                    TokenizeState::Complete(args) => {
                        buffer.clear();

                        if args.is_empty() {
                            continue;
                        }

                        match execute(&args) {
                            Status::Continue => {}
                            Status::Exit(code) => {
                                exit_code = code;
                                break;
                            }
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("0-shell: read error: {}", error);
                break;
            }
        }
    }

    process::exit(exit_code);
}
