use crate::commands::Status;
use std::env;

pub fn run(args: &[String]) -> Result<Status, String> {
    if !args.is_empty() {
        return Err("pwd: too many arguments".to_string());
    }
    let path = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
    println!("{}", path.display());
    Ok(Status::Continue)
}
