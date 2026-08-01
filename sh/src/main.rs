use sh::{Status, execute, tokenize};
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("$ ");
        let _ = io::stdout().flush();

        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\n', '\r']);
                if trimmed.trim().is_empty() {
                    continue;
                }

                match tokenize(trimmed) {
                    Ok(args) => {
                        if args.is_empty() {
                            continue;
                        }
                        if execute(&args) == Status::Exit {
                            break;
                        }
                    }
                    Err(e) => eprintln!("0-shell: {}", e),
                }
            }
            Err(error) => {
                eprintln!("0-shell: read error: {}", error);
                break;
            }
        }
    }
}
