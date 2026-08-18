use crate::commands::{expand_tilde, set_previous_dir, take_previous_dir, Status};
use std::env;
use std::path::Path;

pub fn run(args: &[String]) -> Result<Status, String> {
    let mut newargs: Vec<String> = Vec::new();
    for c in args{
        if c=="--"{
        continue;
        }else{
            newargs.push(c.clone())
        }
    }
    let target = match newargs.len() {
        0 => env::var("HOME").unwrap_or_else(|_| "/".to_string()),
       1 if args[0] == "-" => take_previous_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                env::var("HOME").unwrap_or_else(|_| "/".to_string())
            }),
        1 => expand_tilde(&newargs[0]),
        _ => return Err("cd: too many arguments".to_string()),
    };

    let current_dir = env::current_dir().map_err(|e| format!("cd: {}", e))?;
    env::set_current_dir(Path::new(&target))
        .map_err(|e| format!("cd: {}: {}", target, e))?;
    set_previous_dir(current_dir);

    if newargs.len() == 1 && newargs[0] == "-" {
        println!("{}", target);
    }

    Ok(Status::Continue)
}
