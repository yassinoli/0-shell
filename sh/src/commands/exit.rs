use crate::commands::Status;

pub fn run(args: &[String]) -> Result<Status, String> {
    if let Some(first_args) = args.first() {
        let code = first_args
            .parse::<i32>()
            .map_err(|_| "Invalid exit status".to_string())?;
        return Ok(Status::Exit(code));
    }

    Ok(Status::Exit(0))
}
