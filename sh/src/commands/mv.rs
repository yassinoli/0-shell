use crate::commands::Status;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn run(args: &[String]) -> Result<Status, String> {
    if args.len() < 2 {
        return Err("mv: missing file operand".to_string());
    }

    let dest = Path::new(args.last().unwrap());
    let sources = &args[..args.len() - 1];

    let dest_is_dir = dest.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        return Err(format!(
            "mv: target '{}': Not a directory",
            dest.display()
        ));
    }

    for src in sources {
        let src_path = Path::new(src);
        let target: PathBuf = if dest_is_dir {
            match src_path.file_name() {
                Some(name) => dest.join(name),
                None => {
                    eprintln!("mv: cannot move '{}': Invalid path", src);
                    continue;
                }
            }
        } else {
            dest.to_path_buf()
        };

        if let Err(e) = move_path(src_path, &target) {
            eprintln!("mv: cannot move '{}' to '{}': {}", src, target.display(), e);
        }
    }

    Ok(Status::Continue)
}

fn move_path(src: &Path, dest: &Path) -> std::io::Result<()> {
    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(18) || e.kind() == ErrorKind::CrossesDevices => {
            // EXDEV: cross-device link — copy then remove
            if src.is_dir() {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "cannot move directories across devices",
                ));
            }
            fs::copy(src, dest)?;
            fs::remove_file(src)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
