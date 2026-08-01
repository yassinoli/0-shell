use crate::commands::Status;
use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    if args.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    for path in args {
        if let Err(e) = fs::create_dir(Path::new(path)) {
            eprintln!("mkdir: cannot create directory '{}': {}", path, e);
        }
    }
    Ok(Status::Continue)
}
