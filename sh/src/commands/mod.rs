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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Continue,
    Exit,
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
