use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match stdin.read_line(&mut line) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let command = line.trim().split_whitespace().next().unwrap_or("");
                if !command.is_empty() {
                    println!("command: {}", command);
                }
                line.clear();
            }
            Err(error) => {
                eprintln!("read error: {}", error);
                break;
            }
        }
    }
}
