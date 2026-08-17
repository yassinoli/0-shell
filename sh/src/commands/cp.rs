use crate::commands::{expand_tilde, Status};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(args: &[String]) -> Result<Status, String> {
    if args.len() < 2 {
        return Err("cp: missing file operand".to_string());
    }

    let dest_str = expand_tilde(args.last().unwrap());
    let dest = Path::new(&dest_str);
    let sources = &args[..args.len() - 1];

    let dest_is_dir = dest.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        return Err(format!(
            "cp: target '{}': Not a directory",
            dest.display()
        ));
    }

    for src in sources {
        let expanded_src = expand_tilde(src);
        let src_path = Path::new(&expanded_src);
        if src_path.is_dir() {
            eprintln!("cp: -r not specified; omitting directory '{}'", expanded_src);
            continue;
        }

        let target: PathBuf = if dest_is_dir {
            match src_path.file_name() {
                Some(name) => dest.join(name),
                None => {
                    eprintln!("cp: cannot copy '{}': Invalid path", expanded_src);
                    continue;
                }
            }
        } else {
            dest.to_path_buf()
        };

        if let Err(e) = fs::copy(src_path, &target) {
            eprintln!("cp: cannot copy '{}' to '{}': {}", expanded_src, target.display(), e);
        }
    }

    Ok(Status::Continue)
}
