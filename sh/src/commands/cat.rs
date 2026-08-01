use crate::commands::Status;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    if args.is_empty() {
        return Err("cat: missing operand".to_string());
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut had_error = false;

    for path in args {
        if path == "-" {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();
            if let Err(e) = io::copy(&mut stdin, &mut out) {
                eprintln!("cat: -: {}", e);
                had_error = true;
            }
            continue;
        }

        match File::open(Path::new(path)) {
            Ok(mut file) => {
                let mut buf = Vec::new();
                if let Err(e) = file.read_to_end(&mut buf) {
                    eprintln!("cat: {}: {}", path, e);
                    had_error = true;
                    continue;
                }
                if let Err(e) = out.write_all(&buf) {
                    eprintln!("cat: {}: {}", path, e);
                    had_error = true;
                }
            }
            Err(e) => {
                eprintln!("cat: {}: {}", path, e);
                had_error = true;
            }
        }
    }

    let _ = out.flush();
    if had_error {
        // Keep shell alive; errors already printed.
    }
    Ok(Status::Continue)
}
