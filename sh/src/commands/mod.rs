pub mod cat;
pub mod cd;
pub mod cp;
pub mod echo;
pub mod exit;
pub mod ls;
pub mod mkdir;
pub mod mv;
pub mod pwd;
pub mod rm;
use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static PREVIOUS_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

pub(crate) fn previous_dir() -> &'static Mutex<Option<PathBuf>> {
    PREVIOUS_DIR.get_or_init(|| Mutex::new(None))
}

pub(crate) fn set_previous_dir(path: PathBuf) {
    if let Ok(mut previous) = previous_dir().lock() {
        *previous = Some(path);
    }
}

pub(crate) fn take_previous_dir() -> Option<PathBuf> {
    previous_dir().lock().ok().and_then(|previous| previous.clone())
}

pub fn expand_tilde(path: &str) -> String {
    if path == "~" ||  path == "$HOME"  {
        return env::var("HOME").unwrap_or_else(|_| "/".to_string());
    }

    if let Some(rest) = path.strip_prefix("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        return format!("{}/{}", home, rest);
    }

    path.to_string()
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Continue,
    Exit(i32),
}

pub fn execute(args: &[String]) -> Status {
    if args.is_empty() {
        return Status::Continue;
    }

    let cmd = args[0].as_str();
    let rest = &args[1..];

    let result = match cmd {
        "echo" => echo::run(rest),
        "cd" => cd::run(rest),
        "ls" => ls::run(rest),
        "pwd" => pwd::run(rest),
        "cat" => cat::run(rest),
        "cp" => cp::run(rest),
        "rm" => rm::run(rest),
        "mv" => mv::run(rest),
        "mkdir" => mkdir::run(rest),
        "exit" => exit::run(rest),
        _ => {
            eprintln!("Command '{}' not found", cmd);
            Ok(Status::Continue)
        }
    };

    match result {
        Ok(status) => status,
        Err(e) => {
            eprintln!("{}", e);
            Status::Continue
        }
    }
}
