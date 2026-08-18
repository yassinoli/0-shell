use crate::commands::{expand_tilde, Status};
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

    let mut skipp = false ;
    for arg in args {
        if arg == "--" {
            skipp = true;
            continue;
        }
        if arg.starts_with('-') && arg.len() > 1 && arg != "-" && skipp == false {
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
        let expanded = expand_tilde(p);
        let path = Path::new(&expanded);
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
                eprintln!("ls: cannot access '{}': {}", expanded, e);
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

    list_named_entries(&names, &paths, flags,true)
}

// Prepare explicit file paths for output
fn list_entries(paths: &[PathBuf], flags: &Flags, is_dir: bool) -> io::Result<()> {
    // Extract display names from file paths
    let names: Vec<String> = paths
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect();
    list_named_entries(&names, paths, flags,is_dir)
}

// Print entries using either detailed (-l) or standard layout
fn list_named_entries(names: &[String], paths: &[PathBuf], flags: &Flags, is_dir: bool) -> io::Result<()> {
    if flags.long {
        print_long(names, paths, flags , is_dir)?;
    } else {
        print_short(names, paths, flags)?;
    }
    Ok(())
}

fn display_name(name: &str) -> String {
    let mut result = String::new();

    for c in name.chars() {
        match c {
            '\n' => result.push_str("'$'\\n''"),
            '\t' => result.push_str("'$'\\t''"),
            '\r' => result.push_str("'$'\\r''"),
            '\\' => result.push_str("\\\\"),
            _ => result.push(c),
        }
    }

    result
}
fn print_short(names: &[String], paths: &[PathBuf], flags: &Flags) -> io::Result<()> {
    let stdout = io::stdout();
      // Lock stdout so we can write to it safely and efficiently
    let mut out = stdout.lock();
     // Go through every file/directory name
    for (i, name) in names.iter().enumerate() {
         // If -F is used, add a symbol to the name:
        // / for directory, * for executable, @ for symlink, ........
        let display = if flags.classify {
            classify_name(name, &paths[i])
        } else {
            name.clone()
        };
         let display = display_name(&display);
        writeln!(out, "{}", display)?;
    }
    Ok(())
}

fn print_long(names: &[String], paths: &[PathBuf], flags: &Flags , is_dir:bool) -> io::Result<()> {
     // Store metadata for each file/directory.
    let mut metas: Vec<Option<Metadata>> = Vec::with_capacity(paths.len());
    // Count the total number of filesystem blocks.
    let mut total_blocks: u64 = 0;

    for path in paths {
         // symlink_metadata() does NOT follow symbolic links.
        match fs::symlink_metadata(path) {
            Ok(m) => {
                // Add the number of filesystem blocks used by this file
                total_blocks += m.blocks();
                metas.push(Some(m));
            }
            Err(_) => metas.push(None),
        }
    }
    if is_dir{
       println!("total {}", total_blocks/2);
    }
    
    // align the output.
    let mut link_w = 1usize;
    let mut user_w = 1usize;
    let mut group_w = 1usize;
    let mut size_w = 1usize;

    // Store the information that will be printed for each file
    let mut rows: Vec<Option<LongRow>> = Vec::with_capacity(paths.len());

    // Build one LongRow for each path
    for (i, path) in paths.iter().enumerate() {
        let Some(ref meta) = metas[i] else {
            rows.push(None);
            continue;
        };
         // Convert metadata into information needed by "ls -l"
        // such as permissions, owner, group, size, date, and name.
        let row = build_long_row(&names[i], path, meta, flags);

        // Find the largest width of each column.
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
            row.mode,      // Permissions: -rwxr-xr-x...
            row.nlink_str, // Number of hard links
            row.user,      // Owner
            row.group,     // Group
            row.size_str,  // File size
            row.mtime,     // Modification time
            row.name,      // File name
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
    let mode = format_mode(meta);               // Get the file type and permissions.
    let nlink_str = meta.nlink().to_string();   // Get the number of hard links.
    let user = resolve_user(meta.uid());        // Convert the user ID (UID) into a username.
    let group = resolve_group(meta.gid());      // Convert the group ID (GID) into a group name.
    let size_str = format_size(meta);           // Get the file size. - For normal files: size in bytes. -For devices: major, minor numbers.
    let mtime = format_mtime(meta);             // Get the modification date/time.

    let name = if meta.file_type().is_symlink() {
        match fs::read_link(path) {
            Ok(target) => {
                let target_name = if flags.classify {
                    classify_target(&target, path)
                } else {
                    target.display().to_string()
                };
                format!("{} -> {}", name, target_name)
            }
            Err(_) => name.to_string(),
        }
    } else {
        if flags.classify {
            classify_name(name, path)
        } else {
            name.to_string()
        }
    };

    // Put all the collected information into one LongRow.
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
    // 1 for file type 9  for permissions
    let mut s = String::with_capacity(10);
    // 1 for file type
    s.push(if ft.is_dir() {
        'd' //directory
    } else if ft.is_symlink() {
        'l' //symbolic link
    } else if ft.is_char_device() {
        'c' //character device
    } else if ft.is_block_device() {
        'b' //block device
    } else if ft.is_fifo() {
        'p' // FIFO  
    } else if ft.is_socket() {
        's' //Unix socket
    } else {
        '-' //regular file
    });

    let mode = meta.permissions().mode();
    // Owner permissions  - 0o400 = owner can read
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });

    // 0o200 = owner can write
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });

    // 0o100 = owner can execute - 0o4000 = setuid
    // special_exec() handles normal execute + setuid.
    s.push(special_exec(mode, 0o100, 0o4000, 's', 'S'));

    // Group permissions - 0o040 = group can read
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });

    // 0o020 = group can write
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });

    // 0o010 = group can execute - 0o2000 = setgid
    s.push(special_exec(mode, 0o010, 0o2000, 's', 'S'));

    // Others permissions -  0o004 = others can read
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });

    // 0o002 = others can write
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });

    // 0o001 = others can execute - 0o1000 = sticky bit
    s.push(special_exec(mode, 0o001, 0o1000, 't', 'T'));

    s
}

fn special_exec(mode: u32, exec_bit: u32, special_bit: u32, lower: char, upper: char) -> char {
    // Check if the execute permission is enabled.
    let exec = mode & exec_bit != 0;
    // Check if the special permission is enabled.
    let special = mode & special_bit != 0;
     // Decide which character to display.
    match (exec, special) {
        (true, true) => lower,  // Execute + special permissio 
        (false, true) => upper, // Special permission exists, but execute is disabled.
        (true, false) => 'x', // Execute permission only
        (false, false) => '-', // Neither execute nor special permission
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

/// Convert Unix timestamp to calendar date (UTC)
fn civil_from_days(timestamp: i64) -> (u32, u32, u32, i32, i32, i32) {
    let days = timestamp.div_euclid(86400);
    let tod = timestamp.rem_euclid(86400) as u32;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;

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
    classify_display(name, &meta, true)
}

fn classify_target(target: &Path, link_path: &Path) -> String {
    let meta = match fs::metadata(link_path) {
        Ok(m) => m,
        Err(_) => return target.display().to_string(),
    };
    classify_display(&target.display().to_string(), &meta, false)
}

fn classify_display(name: &str, meta: &Metadata, include_symlink_marker: bool) -> String {
    let ft = meta.file_type();
    if include_symlink_marker && ft.is_symlink() {
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

// 
fn resolve_user(uid: u32) -> String {
    users().get(&uid).cloned().unwrap_or_else(|| uid.to_string())
}

fn resolve_group(gid: u32) -> String {
    groups().get(&gid).cloned().unwrap_or_else(|| gid.to_string())
}
// onelock : A synchronization primitive which can nominally be written to only once.
fn users() -> &'static HashMap<u32, String> {
    static USERS: OnceLock<HashMap<u32, String>> = OnceLock::new();
    USERS.get_or_init(|| parse_id_file("/etc/passwd", 0, 2))
}

fn groups() -> &'static HashMap<u32, String> {
    static GROUPS: OnceLock<HashMap<u32, String>> = OnceLock::new();
    GROUPS.get_or_init(|| parse_id_file("/etc/group", 0, 2))
}


// Read an ID file such as /etc/passwd or /etc/group and create a map from ID to name.
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
