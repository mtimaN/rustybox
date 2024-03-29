extern crate regex;
extern crate chrono;

use regex::Regex;
use std::fs::{set_permissions, Permissions};
use std::os::unix::prelude::PermissionsExt;
use std::{env, fs, path, path::Path};
use std::io::{Read, Write, BufReader, BufRead};
use std::process::ExitCode;
use std::os::unix::fs::{symlink, MetadataExt};
use chrono::{DateTime, Timelike, Datelike, Local};

const INVALID_ERR: i8 = -1;
const PWD_ERR: i8 = -2;
const _ECHO_ERR: i8 = -10;
const CAT_ERR: i8 = -20;
const MKDIR_ERR: i8 = -30;
const MV_ERR: i8 = -40;
const LN_ERR: i8 = -50;
const RMDIR_ERR: i8 = -60;
const RM_ERR: i8 = -70;
const LS_ERR: i8 = -80;
const CP_ERR: i8 = -90;
const TOUCH_ERR: i8 = -100;
const CHMOD_ERR: i8 = -25;
const GREP_ERR: i8 = -5;

fn pwd() -> Result<(), i8> {
    // I found "if let" on the web; it's an alternative to match, suitable
    // for a small number of switch cases.
    if let Ok(current_dir) = env::current_dir() {
        println!("{}", current_dir.display());
        Ok(())
    } else {
        println!("pwd failed!");
        Err(PWD_ERR)
    }
}

fn echo(echo_args: Vec<String>) -> Result<(), i8> {
    // I know it should return -10 on error, but I don't see any way for
    // echo to fail
    if echo_args.len() > 0 {
        let contains_invalid_option = echo_args.iter().any(|arg| {
            arg.starts_with("-") && !arg.eq("-n")
        });

        if contains_invalid_option {
            return Err(INVALID_ERR);
        }

        if echo_args[0] == "-n" {
            if echo_args.len() > 1 {
                print!("{}", echo_args[1]);

                for echo_arg in echo_args[2..].iter() {
                    print!(" {}", echo_arg);
                }
            }
        } else {
            print!("{}", echo_args[0]);
            for echo_arg in echo_args[1..].iter() {
                print!(" {}", echo_arg);
            }
            println!("");
        }
        Ok(())
    } else {
        println!("");
        Ok(())
    }
}

fn cat(cat_args: Vec<String>) -> Result<(), i8> {
    for file_path in cat_args {
        let mut file = fs::File::open(file_path).map_err(|_| CAT_ERR)?;
        let mut cat_content = String::new();

        file.read_to_string(&mut cat_content).map_err(|_| CAT_ERR)?;
        print!("{}", cat_content);
    }
    Ok(())
}

fn mkdir(mkdir_args: Vec<String>) -> Result<(), i8> {
    let mut ret = Ok(());

    for dir in mkdir_args {
        if let Err(_) = fs::create_dir(dir) {
            ret = Err(MKDIR_ERR);
        }
    }
    ret
}

fn mv(mv_args: Vec<String>) -> Result<(), i8> {
    if mv_args.len() < 2 {
        return Err(MV_ERR);
    }
    // mv_args.last() can't fail, so unwrapping is safe.
    let destination = mv_args.last().unwrap();

    // if file exists
    if let Ok(metadata) = fs::metadata(destination) {
        // if it is a directory then all arguments are to be moved there
        if metadata.is_dir() {
            for source in mv_args.iter().take(mv_args.len() - 1) {
                let new_name = format!("{}/{}", destination, source);

                fs::rename(source, new_name).map_err(|_| MV_ERR)?;
            }
        } else {
            if mv_args.len() != 2 {
                return Err(MV_ERR);
            }

            let source = &mv_args[0];
            fs::rename(source, destination).map_err(|_| MV_ERR)?;
        }
    } else {
        if mv_args.len() != 2 {
            return Err(MV_ERR);
        }

        let source = &mv_args[0];
        fs::rename(source, destination).map_err(|_| MV_ERR)?;
    }
    Ok(())
}

fn ln(ln_args: Vec<String>) -> Result<(), i8> {
    if ln_args.len() < 2 {
        return Err(LN_ERR);
    }
    // symbolic link
    if ln_args[0] == "-s" || ln_args[0] == "--symbolic" {
        if ln_args.len() != 3 {
            return Err(LN_ERR);
        }
        let source = &ln_args[1];
        let destination = &ln_args[2];
        symlink(source, destination).map_err(|_| LN_ERR)?;
    } else if ln_args[0].starts_with("-") {
        // any other flag is invalid
        return Err(INVALID_ERR);
    } else {
        // hard link
        if ln_args.len() != 2 {
            return Err(LN_ERR);
        }
        let source = &ln_args[0];
        let destination = &ln_args[1];
        fs::hard_link(source, destination).map_err(|_| LN_ERR)?;
    }
    Ok(())
}

fn rmdir(rmdir_args: Vec<String>) -> Result<(), i8> {
    if rmdir_args.len() == 0 {
        return Err(INVALID_ERR);
    }

    let mut res = Ok(());
    for dir in rmdir_args {
        if let Err(_) = fs::remove_dir(dir) {
            res = Err(RMDIR_ERR);
        }
    }
    res
}

fn rm(rm_args: Vec<String>) -> Result<(), i8> {
    if rm_args.len() == 0 {
        return Err(INVALID_ERR);
    }
    
    let mut res = Ok(());
    if rm_args[0] == "-r" || rm_args[0] == "--recursive" || rm_args[0] == "-R" {
        if rm_args.len() == 1 {
            return Err(INVALID_ERR);
        }
        for file in rm_args[1..].to_vec() {
            if file == "-d" {
                continue;
            }
            if let Ok(metadata) = fs::metadata(&file) {
                if metadata.is_dir() {
                    if let Err(_) = fs::remove_dir_all(file) {
                        res = Err(RM_ERR);
                    }
                } else {
                    if let Err(_) = fs::remove_file(file) {
                        res = Err(RM_ERR);
                    }
                }
            } else {
                res = Err(RM_ERR);
            }
        }
    } else if rm_args[0] == "--dir" || rm_args[0] == "-d" {
        res = rmdir(rm_args[1..].to_vec());
    } else {
        for file in rm_args {
            if let Err(_) = fs::remove_file(file) {
                res = Err(RM_ERR);
            }
        }
    }
    res
}

fn permissions_format(perm: u32) -> &'static str {
    match perm {
        0 => "---",
        1 => "--x",
        2 => "-w-",
        3 => "-wx",
        4 => "r--",
        5 => "r-x",
        6 => "rw-",
        7 => "rwx",
        _ => "---",
    }
}

fn get_username_by_uid(target_uid: u32) -> Result<String, i8> {
    let passwd_file = fs::File::open("/etc/passwd").map_err(|_| LS_ERR)?;
    let reader = BufReader::new(passwd_file);

    for line in reader.lines() {
        let line = line.map_err(|_| LS_ERR)?;
        let fields: Vec<&str> = line.split(':').collect();
        if let [_, _, uid, _, _, _, _] = &fields[..] {
            if let Ok(parsed_uid) = uid.parse::<u32>() {
                if parsed_uid == target_uid {
                    return Ok(fields[0].to_string());
                }
            }
        }
    }

    Err(LS_ERR)
}

fn get_groupname_by_gid(target_gid: u32) -> Result<String, i8> {
    let group_file = fs::File::open("/etc/group").map_err(|_| LS_ERR)?;
    let reader = BufReader::new(group_file);

    for line in reader.lines() {
        let line = line.map_err(|_| LS_ERR)?;
        let fields: Vec<&str> = line.split(':').collect();
        if let [_, _, gid, _] = &fields[..] {
            if let Ok(parsed_gid) = gid.parse::<u32>() {
                if parsed_gid == target_gid {
                    return Ok(fields[0].to_string());
                }
            }
        }
    }

    Err(LS_ERR)
}

fn format_last_modify_time(metadata: fs::Metadata) -> Result<String, i8> {
    let last_modify_time = metadata.modified().map_err(|_| LS_ERR)?;

    let date_time: DateTime<Local> = last_modify_time.into();
    // the month is left out for some reason
    let formatted_time = format!(
        "{} {:02}:{:02}",
        date_time.day(),
        date_time.hour(),
        date_time.minute()
    );

    Ok(formatted_time)
}

fn long_print(entry: path::PathBuf) -> Result<(), i8> {
    let file_name = entry.file_name().unwrap();
    let file_name_str = file_name.to_string_lossy();

    let mut long_string = String::new();
    let metadata = match entry.metadata() {
        Ok(metadata) => metadata,
        Err(_) => {
            return Err(LS_ERR);
        }
    };
    if metadata.is_dir() {
        long_string.push('d');
    } else if metadata.is_symlink() {
        long_string.push('l');
    } else {
        long_string.push('-');
    }
    // get the 3 permission octals
    let permissions = metadata.permissions().mode() % 512;
    let permissions = permissions / 64 * 100 + permissions % 64 / 8 * 10 + permissions % 8;

    // permissions
    long_string.push_str(permissions_format(permissions / 100));
    long_string.push_str(permissions_format(permissions / 10 % 10));
    long_string.push_str(permissions_format(permissions % 10));
    long_string.push(' ');

    // owner
    let owner_id = metadata.uid();
    let owner = get_username_by_uid(owner_id)?;
    long_string.push_str(&owner);

    // group
    let group_id = metadata.gid();
    let group = get_groupname_by_gid(group_id)?;
    long_string.push(' ');
    long_string.push_str(&group);

    // size
    long_string.push(' ');
    long_string.push_str(&metadata.len().to_string());

    // time
    let time = format_last_modify_time(metadata)?;
    long_string.push(' ');
    long_string.push_str(&time);

    // name
    long_string.push(' ');
    long_string.push_str(&file_name_str);

    println!("{}", long_string);
    Ok(())

}

fn ls_r(dirs: Vec<String>, show_hidden: bool, long: bool) -> Result<(), i8> {
    let mut res = Ok(());
    for dir in dirs.iter() {
            println!("{}:", dir);
            let mut recursive_dir = vec![];
            if show_hidden {
                println!(".\n..");
            }
            let entries: Vec<_> = fs::read_dir(dir)
            .map_err(|_| LS_ERR)?
            .map(|entry| entry.expect("Failed to read entry").path())
            .collect();
            let mut sorted_entries = entries;

            // ignoring uppercase while sorting
            sorted_entries.sort_by(|a, b| {
                let a_name = a.file_name().unwrap().to_string_lossy().to_lowercase();
                let b_name = b.file_name().unwrap().to_string_lossy().to_lowercase();
                a_name.cmp(&b_name)
            });
            for entry in sorted_entries {
                let path = entry.as_path();
                let file_name = path.file_name().ok_or(LS_ERR)?;
                let file_name_string = file_name.to_string_lossy().to_string();
                if file_name_string.starts_with(".") && !show_hidden {
                    continue;
                }

                if path.is_dir() {
                    recursive_dir.push(path.to_string_lossy().to_string());
                }
                
                if long {
                    res = long_print(entry);
                } else {
                    println!("{}", file_name_string);
                }
            }

            println!("");
            if recursive_dir.len() > 0 {
                res = ls_r(recursive_dir, show_hidden, long);
            }
    }
    res
}

fn ls(mut ls_args: Vec<String>) -> Result<(), i8> {
    // just learned about contains :)
    let recursive = ls_args.contains(&String::from("-R")) || 
                          ls_args.contains(&String::from("--recursive"));
    
    let long = ls_args.contains(&String::from("-l"));
    let show_hidden = ls_args.contains(&String::from("-a"));
    // check if any directory is given
    let no_dirs = ls_args.iter().all(|arg| arg.starts_with("-"));

    let mut res = Ok(());
    // add the current dir if none are given
    if no_dirs {
        if let Ok(curr_dir) = env::current_dir() {
            ls_args.push(curr_dir.display().to_string());
        } else {
            return Err(LS_ERR);
        }
    }

    if recursive {
        ls_args.retain(|s| !s.starts_with("-"));
        return ls_r(ls_args, show_hidden, long);
    }

    for dir in ls_args.iter().filter(|&arg| !arg.starts_with("-")) {
        let path = Path::new(dir);
        if path.is_file() {
            println!("{}", dir);
            continue;
        }
        if show_hidden {
            println!(".\n..");
        }
        let entries: Vec<_> = fs::read_dir(dir)
        .map_err(|_| LS_ERR)?
        .map(|entry| entry.expect("Failed to read entry").path())
        .collect();
        let mut sorted_entries = entries;

        // ignoring uppercase while sorting
        sorted_entries.sort_by(|a, b| {
            let a_name = a.file_name().unwrap().to_string_lossy().to_lowercase();
            let b_name = b.file_name().unwrap().to_string_lossy().to_lowercase();
            a_name.cmp(&b_name)
        });
        for entry in sorted_entries {
            let file_name = entry.file_name().unwrap();
            let file_name_str = file_name.to_string_lossy();

            // skip hidden files unless -a flag is present
            if !show_hidden && file_name_str.starts_with('.') {
                continue;
            }
            if long {
                res = long_print(entry);
            } else {
                println!("{}", file_name_str);
            }
        }
    }
    res
}

fn cp(cp_args: Vec<String>) -> Result<(), i8> {
    if cp_args.len() < 2 {
        return Err(CP_ERR);
    }
    let recursive = cp_args[0] == "-r" ||
                          cp_args[0] == "-R" ||
                          cp_args[0] == "--recursive";
    let src = if recursive { &cp_args[1] } else { &cp_args[0] };
    let dest = if recursive { &cp_args[2] } else { &cp_args[1] };
    let src_path = Path::new(src);
    if src_path.is_file() {
        if Path::new(dest).is_dir() {
            if let Some(file_name) = src_path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let new_dest = format!("{}/{}", dest, file_name_str);
                    if let Ok(_) = fs::copy(src, new_dest) {
                        return Ok(());
                    } else {
                        return Err(CP_ERR);
                    }
                } else {
                    return Err(CP_ERR);
                }
            } else {
                return Err(CP_ERR);
            }
        } else if let Ok(_) = fs::copy(src, dest) {
            return Ok(());
        } else {
            return Err(CP_ERR);
        }
    } else if recursive && Path::new(src).is_dir() {
        let mut new_dest = dest.clone();

        // copying the whole directory rather than the contents of it
        if !new_dest.contains("/") {
            new_dest = format!("{}/{}", dest, src);
        }

        if !Path::new(&new_dest).exists() {
            fs::create_dir_all(&new_dest).map_err(|_| CP_ERR)?;
        }
        if let Ok(entries) = fs::read_dir(src) {
            let mut res = Ok(());
            for entry in entries {
                let entry = entry.as_ref().map_err(|_| CP_ERR)?;
                let entry_path = entry.path();
                let file_name = entry_path.file_name().unwrap().to_string_lossy().to_string();
                let dest_path = format!("{}/{}", new_dest, file_name);

                let mut recursive_args = vec!("-r".to_string());
                recursive_args.push(entry_path.display().to_string());
                recursive_args.push(dest_path);
                res = cp(recursive_args);
            }
            return res;
        } else {
            return Err(CP_ERR);
        }
    } else {
        return Err(CP_ERR);
    }
}

fn touch(touch_args: Vec<String>) -> Result<(), i8> {
    let mut res = Ok(());
    let no_create = touch_args.contains(&String::from("-c")) ||
                          touch_args.contains(&String::from("--no-create"));

    let only_access = touch_args.contains(&String::from("-a"));
    let only_modify = touch_args.contains(&String::from("-m"));

    for arg in touch_args.iter().filter(|&arg| !arg.starts_with("-")) {
        if let Ok(mut file) = fs::OpenOptions::new().read(true)
                                      .append(true)
                                      .create(!no_create)
                                      .open(arg) {
            if !only_modify {
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);
            }
            if !only_access {
                let write_buf = [b'\0'];
                let _ = file.write(&write_buf);
            }
        } else {
            // touching a non-existant file with -c is not an error
            if !no_create {
                res = Err(TOUCH_ERR);
            }
        }
    }
    res
}

fn chmod(chmod_args: Vec<String>) -> Result<(), i8> {
    if chmod_args.len() != 2 {
        return Err(INVALID_ERR);
    }
    let contains_flags = chmod_args.iter().any(|flag| flag.starts_with('-') &&
                                                    !(flag.contains("r") || flag.contains("w") || flag.contains("x")));
    if contains_flags {
        return Err(INVALID_ERR);
    }

    if let Ok(perm8) = chmod_args[0].parse::<u32>() {
        // number permissions
        if perm8 > 777 {
            return Err(CHMOD_ERR);
        }
        let perm: u32 = perm8 / 100 * 64 + (perm8 / 10) % 10 * 8 + perm8 % 10;
        set_permissions(&chmod_args[1], fs::Permissions::from_mode(perm)).map_err(|_| CHMOD_ERR)?;
    } else {
        // string perms
        let old_perm = fs::metadata(&chmod_args[1]).map_err(|_| CHMOD_ERR)?;

        let mut perm: Permissions = Permissions::from_mode(0);
        let mut chars = chmod_args[0].chars();

        let mut plus = true;
        let mut user = 0;
        while let Some(c) = chars.next() {
            match c {
                'a' => user = 0o777,
                'u' => user = user | 0o700,
                'g' => user = user | 0o070,
                'o' => user = user | 0o007,
                '+' => plus = true,
                '-' => plus = false,
                'r' => perm.set_mode(perm.mode() | 0o444),
                'w' => perm.set_mode(perm.mode() | 0o222),
                'x' => perm.set_mode(perm.mode() | 0o111),
                _ => return Err(CHMOD_ERR),
            }
        }
        if user == 0 {
            user = 0o777;
        }
        perm.set_mode(perm.mode() & user);
        if plus {
            perm.set_mode(old_perm.mode() | perm.mode());
        } else {
            perm.set_mode(old_perm.mode() & !perm.mode());
        }

        set_permissions(&chmod_args[1], perm).map_err(|_| CHMOD_ERR)?;
    }
    Ok(())
}

fn grep(grep_args: Vec<String>) -> Result<(), i8> {
    let invert = grep_args[0] == "-i";
    let regex_pattern = if invert { &grep_args[1] } else { &grep_args[0] };
    let regex = Regex::new(&regex_pattern).map_err(|_| GREP_ERR)?;

    let file_path = if invert { &grep_args[2] } else { &grep_args[1] };
    let file = fs::File::open(file_path).map_err(|_| GREP_ERR)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            if regex.is_match(&line) == !invert {
                println!("{}", line);
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    /* get() attempts to access the first element in the vector
       if it doesn't exist, main doesn't panic.
       map selects the Some() and ignores the None() */
    let return_code = match args.get(1).map(String::as_str) {
        Some("pwd") => pwd(),
        Some("echo") => echo(args[2..].to_vec()),
        Some("cat") => cat(args[2..].to_vec()),
        Some("mkdir") => mkdir(args[2..].to_vec()),
        Some("mv") => mv(args[2..].to_vec()),
        Some("ln") => ln(args[2..].to_vec()),
        Some("rmdir") => rmdir(args[2..].to_vec()),
        Some("rm") => rm(args[2..].to_vec()),
        Some("ls") => ls(args[2..].to_vec()),
        Some("cp") => cp(args[2..].to_vec()),
        Some("touch") => touch(args[2..].to_vec()),
        Some("chmod") => chmod(args[2..].to_vec()),
        Some("grep") => grep(args[2..].to_vec()),
        _ => Err(INVALID_ERR),
    };

    if return_code == Err(INVALID_ERR) {
        println!("Invalid command");
    }

    match return_code {
        Ok(()) => ExitCode::from(0),
        Err(code) => ExitCode::from(code as u8),
    }
}
