use crate::commands::Status;

pub fn run(args: &[String]) -> Result<Status, String> {
    println!("{}", args.join(" "));
    Ok(Status::Continue)
}
