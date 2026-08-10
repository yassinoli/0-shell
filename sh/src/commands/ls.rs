use crate::commands::Status;
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::io::{self, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct Flags {
    long: bool,
    all: bool,
    classify: bool,
}

pub fn run(args: &[String]) -> Result<Status, String> {
    let (flags, paths) = parse_args(args)?;

    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
    };

    let mut files = Vec::new();
    let mut directories = Vec::new();

    for path in paths {
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    directories.push(path);
                } else {
                    files.push(path);
                }
            }

            Err(error) => {
                eprintln!(
                    "ls: cannot access '{}': {}",
                    path.display(),
                    error
                );
            }
        }
    }

    let multiple = files.len() + directories.len() > 1;
    let mut printed_something = false;

    if !files.is_empty() {
        display_entries(&files, &flags)?;

        printed_something = true;
    }

    for directory in directories {
        if printed_something {
            println!();
        }

        if multiple {
            println!("{}:", directory.display());
        }

        display_directory(&directory, &flags)?;
        printed_something = true;
    }

    Ok(Status::Continue)
}

fn parse_args(args: &[String]) -> Result<(Flags, Vec<String>), String> {
    let mut flags = Flags::default();
    let mut paths = Vec::new();

    for arg in args {
        if arg == "--" {
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            for flag in arg.chars().skip(1) {
                match flag {
                    'l' => flags.long = true,
                    'a' => flags.all = true,
                    'F' => flags.classify = true,

                    _ => {
                        return Err(format!(
                            "ls: invalid option -- '{}'",
                            flag
                        ));
                    }
                }
            }
        } else {
            paths.push(arg.clone());
        }
    }

    Ok((flags, paths))
}

fn display_directory(
    directory: &Path,
    flags: &Flags,
) -> io::Result<()> {
    let mut entries = collect_entries(directory, flags)?;

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let names: Vec<String> = entries
        .iter()
        .map(|entry| entry.0.clone())
        .collect();

    let paths: Vec<PathBuf> = entries
        .iter()
        .map(|entry| entry.1.clone())
        .collect();

    display_entries_with_names(&names, &paths, flags)
}

fn collect_entries(
    directory: &Path,
    flags: &Flags,
) -> io::Result<Vec<(String, PathBuf)>> {
    let mut result = Vec::new();

    if flags.all {
        result.push((".".to_string(), directory.to_path_buf()));
        result.push(("..".to_string(), directory.join("..")));
    }

    for item in fs::read_dir(directory)? {
        let item = item?;
        let name = item.file_name().to_string_lossy().to_string();

        if !flags.all && name.starts_with('.') {
            continue;
        }

        result.push((name, item.path()));
    }

    Ok(result)
}

fn display_entries(
    paths: &[PathBuf],
    flags: &Flags,
) -> io::Result<()> {
    let names: Vec<String> = paths
        .iter()
        .map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        })
        .collect();

    display_entries_with_names(&names, paths, flags)
}

fn display_entries_with_names(
    names: &[String],
    paths: &[PathBuf],
    flags: &Flags,
) -> io::Result<()> {
    if flags.long {
        print_long_format(names, paths, flags)
    } else {
        print_short_format(names, paths, flags)
    }
}

fn print_short_format(
    names: &[String],
    paths: &[PathBuf],
    flags: &Flags,
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();

    for (index, name) in names.iter().enumerate() {
        let value = if flags.classify {
            classify(name, &paths[index])
        } else {
            name.clone()
        };

        writeln!(output, "{}", value)?;
    }

    Ok(())
}

fn print_long_format(
    names: &[String],
    paths: &[PathBuf],
    flags: &Flags,
) -> io::Result<()> {
    let mut rows = Vec::new();
    let mut total_blocks = 0;

    for (name, path) in names.iter().zip(paths.iter()) {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        total_blocks += metadata.blocks();

        rows.push(make_row(
            name,
            path,
            &metadata,
            flags,
        ));
    }

    println!("total {}", total_blocks);

    let link_width = rows
        .iter()
        .map(|row| row.links.len())
        .max()
        .unwrap_or(1);

    let user_width = rows
        .iter()
        .map(|row| row.user.len())
        .max()
        .unwrap_or(1);

    let group_width = rows
        .iter()
        .map(|row| row.group.len())
        .max()
        .unwrap_or(1);

    let size_width = rows
        .iter()
        .map(|row| row.size.len())
        .max()
        .unwrap_or(1);

    let stdout = io::stdout();
    let mut output = stdout.lock();

    for row in rows {
        writeln!(
            output,
            "{} {:>link_width$} {:user_width$} {:group_width$} {:>size_width$} {} {}",
            row.mode,
            row.links,
            row.user,
            row.group,
            row.size,
            row.date,
            row.name,
            link_width = link_width,
            user_width = user_width,
            group_width = group_width,
            size_width = size_width,
        )?;
    }

    Ok(())
}

struct Row {
    mode: String,
    links: String,
    user: String,
    group: String,
    size: String,
    date: String,
    name: String,
}

fn make_row(
    name: &str,
    path: &Path,
    metadata: &Metadata,
    flags: &Flags,
) -> Row {
    let display_name = if flags.classify {
        classify(name, path)
    } else {
        name.to_string()
    };

    let name = if metadata.file_type().is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                format!(
                    "{} -> {}",
                    display_name.trim_end_matches('@'),
                    target.display()
                )
            }

            Err(_) => display_name,
        }
    } else {
        display_name
    };

    Row {
        mode: permissions_string(metadata),
        links: metadata.nlink().to_string(),
        user: get_user(metadata.uid()),
        group: get_group(metadata.gid()),
        size: file_size(metadata),
        date: modified_time(metadata),
        name,
    }
}

fn permissions_string(metadata: &Metadata) -> String {
    let file_type = metadata.file_type();
    let mode = metadata.permissions().mode();

    let first = if file_type.is_dir() {
        'd'
    } else if file_type.is_symlink() {
        'l'
    } else if file_type.is_char_device() {
        'c'
    } else if file_type.is_block_device() {
        'b'
    } else if file_type.is_fifo() {
        'p'
    } else if file_type.is_socket() {
        's'
    } else {
        '-'
    };

    let mut result = String::new();

    result.push(first);

    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(permission_char(
        mode,
        0o100,
        0o4000,
        's',
        'S',
    ));

    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(permission_char(
        mode,
        0o010,
        0o2000,
        's',
        'S',
    ));

    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(permission_char(
        mode,
        0o001,
        0o1000,
        't',
        'T',
    ));

    result
}

fn permission_char(
    mode: u32,
    execute: u32,
    special: u32,
    lower: char,
    upper: char,
) -> char {
    let can_execute = mode & execute != 0;
    let has_special = mode & special != 0;

    match (can_execute, has_special) {
        (true, true) => lower,
        (false, true) => upper,
        (true, false) => 'x',
        (false, false) => '-',
    }
}

fn file_size(metadata: &Metadata) -> String {
    let file_type = metadata.file_type();

    if file_type.is_char_device() || file_type.is_block_device() {
        let device = metadata.rdev();

        let major = (device >> 8) & 0xfff;
        let minor =
            (device & 0xff) |
            ((device >> 12) & 0xfff00);

        format!("{}, {}", major, minor)
    } else {
        metadata.len().to_string()
    }
}

fn modified_time(metadata: &Metadata) -> String {
    let time = metadata
        .modified()
        .unwrap_or(UNIX_EPOCH);

    let timestamp = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let (_, minute, hour, day, month, year) =
        unix_to_date(timestamp);

    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr",
        "May", "Jun", "Jul", "Aug",
        "Sep", "Oct", "Nov", "Dec",
    ];

    let month_name = MONTHS
        .get((month - 1) as usize)
        .unwrap_or(&"???");

    const SIX_MONTHS: i64 = 15778476;

    if (now - timestamp).abs() > SIX_MONTHS {
        format!(
            "{} {:2}  {:4}",
            month_name,
            day,
            year
        )
    } else {
        format!(
            "{} {:2} {:02}:{:02}",
            month_name,
            day,
            hour,
            minute
        )
    }
}

fn unix_to_date(timestamp: i64) -> (u32, u32, u32, i32, i32, i32) {
    let days = timestamp.div_euclid(86400);
    let seconds = timestamp.rem_euclid(86400) as u32;

    let hour = seconds / 3600;
    let minute = (seconds % 3600) / 60;
    let second = seconds % 60;

    let z = days + 719468;

    let era = if z >= 0 {
        z
    } else {
        z - 146096
    } / 146097;

    let day_of_era =
        (z - era * 146097) as u64;

    let year_of_era =
        (day_of_era
            - day_of_era / 1460
            + day_of_era / 36524
            - day_of_era / 146096)
            / 365;

    let year =
        year_of_era as i64 + era * 400;

    let day_of_year =
        day_of_era
            - (365 * year_of_era
                + year_of_era / 4
                - year_of_era / 100);

    let month_part =
        (5 * day_of_year + 2) / 153;

    let day =
        day_of_year
            - (153 * month_part + 2) / 5
            + 1;

    let month =
        if month_part < 10 {
            month_part + 3
        } else {
            month_part - 9
        };

    let year =
        if month <= 2 {
            year + 1
        } else {
            year
        };

    (
        second,
        minute,
        hour,
        day as i32,
        month as i32,
        year as i32,
    )
}

fn classify(name: &str, path: &Path) -> String {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return name.to_string(),
    };

    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        format!("{}@", name)
    } else if file_type.is_dir() {
        format!("{}/", name)
    } else if file_type.is_fifo() {
        format!("{}|", name)
    } else if file_type.is_socket() {
        format!("{}=", name)
    } else if metadata.permissions().mode() & 0o111 != 0 {
        format!("{}*", name)
    } else {
        name.to_string()
    }
}

fn get_user(uid: u32) -> String {
    user_map()
        .get(&uid)
        .cloned()
        .unwrap_or_else(|| uid.to_string())
}

fn get_group(gid: u32) -> String {
    group_map()
        .get(&gid)
        .cloned()
        .unwrap_or_else(|| gid.to_string())
}

fn user_map() -> &'static HashMap<u32, String> {
    static USERS: OnceLock<HashMap<u32, String>> =
        OnceLock::new();

    USERS.get_or_init(|| {
        read_id_file("/etc/passwd", 0, 2)
    })
}

fn group_map() -> &'static HashMap<u32, String> {
    static GROUPS: OnceLock<HashMap<u32, String>> =
        OnceLock::new();

    GROUPS.get_or_init(|| {
        read_id_file("/etc/group", 0, 2)
    })
}

fn read_id_file(
    filename: &str,
    name_index: usize,
    id_index: usize,
) -> HashMap<u32, String> {
    let mut result = HashMap::new();

    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => return result,
    };

    for line in content.lines() {
        let fields: Vec<&str> =
            line.split(':').collect();

        let max_index = name_index.max(id_index);

        if fields.len() <= max_index {
            continue;
        }

        let id = match fields[id_index].parse::<u32>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        result.insert(
            id,
            fields[name_index].to_string(),
        );
    }

    result
}