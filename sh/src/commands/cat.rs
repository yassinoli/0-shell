use crate::commands::Status;
use std::fs::File;
use std::io::{self, Read, Write , BufRead};
use std::path::Path;

/// Executes a `cat`-like command that processes a list of file paths or standard input (`"-"`)
/// and outputs their contents sequentially to stdout.
///
/// # Arguments
/// * `args` - A slice of string arguments representing target file paths or `"-"` for stdin.
///
/// # Returns
/// * `Ok(Status::Continue)` - Execution finished (even if individual file read errors occurred).
/// * `Err(String)` - Returned if no target arguments are supplied.
pub fn run(args: &[String]) -> Result<Status, String> {
    // Validate that at least one file or stream target is provided
    if args.is_empty() {
      let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(err) => {
                    eprintln!("cat: {}", err);
                    break;
                }
            }
        }
        // sssssssssssssssss
          return Err("cat: missing operand".to_string());
    }

    // Lock standard output once to optimize writing performance across multiple files/streams
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut had_error = false;

    for path in args {
        // Handle standard input (`"-"`)
        if path == "-" {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();

            // Stream data directly from stdin to stdout without reading everything into memory
            if let Err(e) = io::copy(&mut stdin, &mut out) {
                eprintln!("cat: -: {}", e);
                had_error = true;
            }
            continue;
        }

        // Attempt to open the specified file path
        match File::open(Path::new(path)) {
            Ok(mut file) => {
                let mut buf = Vec::new();

                // Read the entire file into a memory buffer
                if let Err(e) = file.read_to_end(&mut buf) {
                    eprintln!("cat: {}: {}", path, e);
                    had_error = true;
                    continue;
                }

                // Write the buffer contents to locked stdout
                if let Err(e) = out.write_all(&buf) {
                    eprintln!("cat: {}: {}", path, e);
                    had_error = true;
                }
            }
            Err(e) => {
                // Handle file opening errors (e.g., File Not Found, Permission Denied)
                eprintln!("cat: {}: {}", path, e);
                had_error = true;
            }
        }
    }

    // Ensure all remaining buffered bytes are written out to stdout
    let _ = out.flush();

    // Log errors were handled via stderr during loop iteration, preserving shell loop execution
    if had_error {
         eprintln!("cat: one or more errors occurred");
    }

    Ok(Status::Continue)
}