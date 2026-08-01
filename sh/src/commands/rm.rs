use crate::commands::Status;
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    let mut recursive = false;
    let mut paths: Vec<&str> = Vec::new();

    for arg in args {
        if arg == "-r" || arg == "-R" || arg == "--recursive" {
            recursive = true;
        } else if arg.starts_with('-') && arg != "-" {
            // Support combined short flags like -rf (only -r matters for us)
            let mut unknown = false;
            for c in arg.chars().skip(1) {
                match c {
                    'r' | 'R' => recursive = true,
                    'f' => {} // ignore force for basic compatibility
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
        let p = Path::new(path);
        let meta = match fs::symlink_metadata(p) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("rm: cannot remove '{}': {}", path, e);
                continue;
            }
        };

        if meta.is_dir() && !meta.file_type().is_symlink() {
            if !recursive {
                eprintln!("rm: cannot remove '{}': Is a directory", path);
                continue;
            }
            if let Err(e) = fs::remove_dir_all(p) {
                eprintln!("rm: cannot remove '{}': {}", path, e);
            }
        } else if let Err(e) = fs::remove_file(p) {
            eprintln!("rm: cannot remove '{}': {}", path, e);
        }
    }

    Ok(Status::Continue)
}
