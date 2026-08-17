use crate::commands::Status;
use std::env;
use std::path::Path;
use crate::commands::{expand_tilde};


pub fn run(args: &[String]) -> Result<Status, String> {
    let target = match args.len() {
        0 => env::var("HOME").unwrap_or_else(|_| "/".to_string()),
        1 => expand_tilde(&args[0]),
        _ => return Err("cd: too many arguments".to_string()),
    };

    env::set_current_dir(Path::new(&target))
        .map_err(|e| format!("cd: {}: {}", target, e))?;
    Ok(Status::Continue)
}