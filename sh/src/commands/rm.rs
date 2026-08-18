use crate::commands::{expand_tilde, Status};
use std::env;
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    let mut recursive = false;
    let mut paths: Vec<&str> = Vec::new();

    // Parse arguments
    for arg in args {
        if arg == "-r" || arg == "-R" || arg == "--recursive" {
            recursive = true;
        } else if arg.starts_with('-') && arg != "-" {
            let mut unknown = false;

            for c in arg.chars().skip(1) {
                match c {
                    'r' | 'R' => recursive = true,
                    'f' => {}
                    _ => {
                        unknown = true;
                        break;
                    }
                }
            }

            if unknown {
                return Err(format!("rm: invalid option -- '{}'", arg));
            }
        } else {
            paths.push(arg);
        }
    }

    if paths.is_empty() {
        return Err("rm: missing operand".to_string());
    }

    for path in paths {
        let expanded = expand_tilde(path);
        let p = Path::new(&expanded);

        let meta = match fs::symlink_metadata(p) {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("rm: cannot remove '{}': {}", expanded, e);
                continue;
            }
        };

        // Directory
        if meta.is_dir() && !meta.file_type().is_symlink() {
            if !recursive {
                eprintln!(
                    "rm: cannot remove '{}': Is a directory",
                    expanded
                );
                continue;
            }

            // IMPORTANT:
            // Resolve the path BEFORE changing the current directory.
            let target = match p.canonicalize() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!(
                        "rm: cannot remove '{}': {}",
                        expanded, e
                    );
                    continue;
                }
            };

            // Get the current directory.
            let current = match env::current_dir() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("rm: cannot remove '{}': {}", expanded, e);
                    continue;
                }
            };

            // Resolve current directory too.
            let current = match current.canonicalize() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("rm: cannot remove '{}': {}", expanded, e);
                    continue;
                }
            };

            // Are we currently inside the directory
            // that we are about to delete?
            if current.starts_with(&target) {
                let home = env::var("HOME")
                    .unwrap_or_else(|_| "/".to_string());

                if let Err(e) = env::set_current_dir(&home) {
                    eprintln!(
                        "rm: cannot change directory to '{}': {}",
                        home, e
                    );
                    continue;
                }
            }

            // Use the absolute path `target`.
            // Do NOT use `p` here because `p` can be relative.
            if let Err(e) = fs::remove_dir_all(&target) {
                eprintln!(
                    "rm: cannot remove '{}': {}",
                    expanded, e
                );
            }
        }

        // File or symlink
        else if let Err(e) = fs::remove_file(p) {
            eprintln!(
                "rm: cannot remove '{}': {}",
                expanded, e
            );
        }
    }

    Ok(Status::Continue)
}