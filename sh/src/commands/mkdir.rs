use crate::commands::{expand_tilde, Status};
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    for path in args {
        let expanded = expand_tilde(path);
        if let Err(e) = fs::create_dir(Path::new(&expanded)) {
            eprintln!("mkdir: cannot create directory '{}': {}", expanded, e);
        }
    }
    Ok(Status::Continue)
}
