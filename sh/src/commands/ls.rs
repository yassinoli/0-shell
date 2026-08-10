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
    let mut flags = Flags::default();
    let mut paths: Vec<&str> = Vec::new();

    for arg in args {
        if arg == "--" {
            continue;
        }
        if arg.starts_with('-') && arg.len() > 1 && arg != "-" {
            for c in arg.chars().skip(1) {
                match c {
                    'l' => flags.long = true,
                    'a' => flags.all = true,
                    'F' => flags.classify = true,
                    _ => return Err(format!("ls: invalid option -- '{}'", c)),
                }
            }
        } else {
            paths.push(arg);
        }
    }
    
    if paths.is_empty() {
        paths.push(".");
    }
    
    let multiple = paths.len() > 1;
    let mut first = true;
    let mut had_error = false;
    
    // Separate files and directories like traditional ls
    let mut files: Vec<PathBuf> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    
    for p in &paths {
        let path = Path::new(p);
        // Fetch metadata without following symlinks
        match fs::symlink_metadata(path) {
            Ok(meta) => {
                // Group actual directories separately from files and symlinks
                if meta.is_dir() && !meta.file_type().is_symlink() {
                    dirs.push(path.to_path_buf());
                } else {
                    files.push(path.to_path_buf());
                }
            }
            Err(e) => {
            // Print access error to stderr and flag exit failure
                eprintln!("ls: cannot access '{}': {}", p, e);
                had_error = true;
            }
        }
    }

        // Print individual files first
    if !files.is_empty() {
        if let Err(e) = list_entries(&files, &flags, false) {
            eprintln!("ls: {}", e);
            had_error = true;
        }
        first = false; // Mark that output has started
    }

    // Process and display each directory
    for dir in &dirs {
      // Print directory headers (e.g. "folder:") and blank separator lines
        if !first || multiple {
            if !first {
                println!();
            }
            println!("{}:", dir.display());
        }
        first = false;

        // List directory contents
        match list_directory(dir, &flags) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("ls: cannot open directory '{}': {}", dir.display(), e);
                had_error = true;
            }
        }
    }

    let _ = had_error;// Silence unused variable compiler warning
    Ok(Status::Continue)
}
// Collect, filter, and display the contents of a directory
fn list_directory(dir: &Path, flags: &Flags) -> io::Result<()> {
    let mut entries: Vec<(String, PathBuf)> = Vec::new();

    // Include '.' and '..' if -a / --all flag is active
    if flags.all {
        entries.push((".".to_string(), dir.to_path_buf()));
        entries.push(("..".to_string(), dir.join("..")));
    }

    // Read items inside the directory
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        // Skip hidden files (starting with '.') unless -a is active
        if !flags.all && name.starts_with('.') {
            continue;
        }
        entries.push((name, entry.path()));
    }

    // Sort items alphabetically by filename
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Separate sorted tuples into parallel name and path vectors
    let paths: Vec<PathBuf> = entries.iter().map(|(_, p)| p.clone()).collect();
    let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();

    list_named_entries(&names, &paths, flags)
}

// Prepare explicit file paths for output
fn list_entries(paths: &[PathBuf], flags: &Flags, _is_dir: bool) -> io::Result<()> {
    // Extract display names from file paths
    let names: Vec<String> = paths
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect();
    list_named_entries(&names, paths, flags)
}

// Print entries using either detailed (-l) or standard layout
fn list_named_entries(names: &[String], paths: &[PathBuf], flags: &Flags) -> io::Result<()> {
    if flags.long {
        print_long(names, paths, flags)?;
    } else {
        print_short(names, paths, flags)?;
    }
    Ok(())
}

fn print_short(names: &[String], paths: &[PathBuf], flags: &Flags) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for (i, name) in names.iter().enumerate() {
        let display = if flags.classify {
            classify_name(name, &paths[i])
        } else {
            name.clone()
        };
        writeln!(out, "{}", display)?;
    }
    Ok(())
}

fn print_long(names: &[String], paths: &[PathBuf], flags: &Flags) -> io::Result<()> {
    let mut metas: Vec<Option<Metadata>> = Vec::with_capacity(paths.len());
    let mut total_blocks: u64 = 0;

    for path in paths {
        match fs::symlink_metadata(path) {
            Ok(m) => {
                // st_blocks is in 512-byte units on Linux
                total_blocks += m.blocks();
                metas.push(Some(m));
            }
            Err(_) => metas.push(None),
        }
    }

    println!("total {}", total_blocks);

    let mut link_w = 1usize;
    let mut user_w = 1usize;
    let mut group_w = 1usize;
    let mut size_w = 1usize;

    let mut rows: Vec<Option<LongRow>> = Vec::with_capacity(paths.len());

    for (i, path) in paths.iter().enumerate() {
        let Some(ref meta) = metas[i] else {
            rows.push(None);
            continue;
        };
        let row = build_long_row(&names[i], path, meta, flags);
        link_w = link_w.max(row.nlink_str.len());
        user_w = user_w.max(row.user.len());
        group_w = group_w.max(row.group.len());
        size_w = size_w.max(row.size_str.len());
        rows.push(Some(row));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for row in rows.into_iter().flatten() {
        writeln!(
            out,
            "{} {:>link_w$} {:user_w$} {:group_w$} {:>size_w$} {} {}",
            row.mode,
            row.nlink_str,
            row.user,
            row.group,
            row.size_str,
            row.mtime,
            row.name,
            link_w = link_w,
            user_w = user_w,
            group_w = group_w,
            size_w = size_w,
        )?;
    }

    Ok(())
}

struct LongRow {
    mode: String,
    nlink_str: String,
    user: String,
    group: String,
    size_str: String,
    mtime: String,
    name: String,
}

fn build_long_row(name: &str, path: &Path, meta: &Metadata, flags: &Flags) -> LongRow {
    let mode = format_mode(meta);
    let nlink_str = meta.nlink().to_string();
    let user = resolve_user(meta.uid());
    let group = resolve_group(meta.gid());
    let size_str = format_size(meta);
    let mtime = format_mtime(meta);
    let display_name = if flags.classify {
        classify_name(name, path)
    } else {
        name.to_string()
    };

    // Symlink target
    let name = if meta.file_type().is_symlink() {
        match fs::read_link(path) {
            Ok(target) => format!("{} -> {}", display_name.trim_end_matches('@'), target.display()),
            Err(_) => display_name,
        }
    } else {
        display_name
    };

    LongRow {
        mode,
        nlink_str,
        user,
        group,
        size_str,
        mtime,
        name,
    }
}

fn format_mode(meta: &Metadata) -> String {
    let ft = meta.file_type();
    let mut s = String::with_capacity(10);

    s.push(if ft.is_dir() {
        'd'
    } else if ft.is_symlink() {
        'l'
    } else if ft.is_char_device() {
        'c'
    } else if ft.is_block_device() {
        'b'
    } else if ft.is_fifo() {
        'p'
    } else if ft.is_socket() {
        's'
    } else {
        '-'
    });

    let mode = meta.permissions().mode();
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(special_exec(mode, 0o100, 0o4000, 's', 'S'));
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(special_exec(mode, 0o010, 0o2000, 's', 'S'));
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(special_exec(mode, 0o001, 0o1000, 't', 'T'));

    s
}

fn special_exec(mode: u32, exec_bit: u32, special_bit: u32, lower: char, upper: char) -> char {
    let exec = mode & exec_bit != 0;
    let special = mode & special_bit != 0;
    match (exec, special) {
        (true, true) => lower,
        (false, true) => upper,
        (true, false) => 'x',
        (false, false) => '-',
    }
}

fn format_size(meta: &Metadata) -> String {
    let ft = meta.file_type();
    if ft.is_char_device() || ft.is_block_device() {
        let rdev = meta.rdev();
        let major = (rdev >> 8) & 0xfff;
        let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
        format!("{}, {}", major, minor)
    } else {
        meta.len().to_string()
    }
}

fn format_mtime(meta: &Metadata) -> String {
    let modified = meta.modified().unwrap_or(UNIX_EPOCH);
    let duration = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Format like: "Feb  5 09:21" or "Feb  5  2024" if older than ~6 months
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let (_sec, min, hour, day, mon, year) = civil_from_days(duration);

    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let mon_str = MONTHS.get((mon - 1) as usize).unwrap_or(&"???");

    let six_months: i64 = 15778476; // ~6 months in seconds
    if (now - duration).abs() > six_months {
        format!("{} {:2}  {:4}", mon_str, day, year)
    } else {
        format!("{} {:2} {:02}:{:02}", mon_str, day, hour, min)
    }
}

/// Convert Unix timestamp to calendar date (UTC). Good enough for ls display.
fn civil_from_days(timestamp: i64) -> (u32, u32, u32, i32, i32, i32) {
    let days = timestamp.div_euclid(86400);
    let tod = timestamp.rem_euclid(86400) as u32;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;

    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (sec, min, hour, d as i32, m as i32, y as i32)
}

fn classify_name(name: &str, path: &Path) -> String {
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return name.to_string(),
    };
    let ft = meta.file_type();
    if ft.is_symlink() {
        format!("{}@", name)
    } else if ft.is_dir() {
        format!("{}/", name)
    } else if ft.is_fifo() {
        format!("{}|", name)
    } else if ft.is_socket() {
        format!("{}=", name)
    } else if meta.permissions().mode() & 0o111 != 0 {
        format!("{}*", name)
    } else {
        name.to_string()
    }
}

fn resolve_user(uid: u32) -> String {
    users().get(&uid).cloned().unwrap_or_else(|| uid.to_string())
}

fn resolve_group(gid: u32) -> String {
    groups().get(&gid).cloned().unwrap_or_else(|| gid.to_string())
}

fn users() -> &'static HashMap<u32, String> {
    static USERS: OnceLock<HashMap<u32, String>> = OnceLock::new();
    USERS.get_or_init(|| parse_id_file("/etc/passwd", 0, 2))
}

fn groups() -> &'static HashMap<u32, String> {
    static GROUPS: OnceLock<HashMap<u32, String>> = OnceLock::new();
    GROUPS.get_or_init(|| parse_id_file("/etc/group", 0, 2))
}

fn parse_id_file(path: &str, name_idx: usize, id_idx: usize) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let Ok(content) = fs::read_to_string(path) else {
        return map;
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > id_idx.max(name_idx)
            && let Ok(id) = parts[id_idx].parse::<u32>()
        {
            map.insert(id, parts[name_idx].to_string());
        }
    }
    map
}
