use crate::commands::Status;

pub fn run(_args: &[String]) -> Result<Status, String> {
    Ok(Status::Exit)
}
