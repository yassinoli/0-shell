use crate::commands::{expand_tilde, set_previous_dir, take_previous_dir, Status};
use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    let target = match args.len() {
        0 => env::var("HOME").unwrap_or_else(|_| "/".to_string()),
        1 if args[0] == "-" => take_previous_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .ok_or_else(|| "cd: OLDPWD not set".to_string())?,
        1 => expand_tilde(&args[0]),
        _ => return Err("cd: too many arguments".to_string()),
    };

    let current_dir = env::current_dir().map_err(|e| format!("cd: {}", e))?;
    env::set_current_dir(Path::new(&target))
        .map_err(|e| format!("cd: {}: {}", target, e))?;
    set_previous_dir(current_dir);

    if args.len() == 1 && args[0] == "-" {
        println!("{}", target);
    }

    Ok(Status::Continue)
}
