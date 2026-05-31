use directories::{BaseDirs};
use std::env::consts::OS;
use std::fs;
use std::path::{Path, PathBuf};

static PARENT_DIR_1: & str = "Wizards Of The Coast";
static PARENT_DIR_2: & str = "MTGA";
static LOG_FILE_NAME: & str = "Player.log";

fn read_file() {
    let path = get_path();
    let read = fs::read_to_string(path);
    println!("FILE: {}", read.unwrap())
    
}

fn get_path() -> PathBuf {
    if let Some(dirs) = BaseDirs::new() {
        let os_dir = match OS {
            "windows" => Path::new("AppData").join("LocalLow"), // This path is untested as not yet run on a windows machine
            "macos" => Path::new("Library").join("Logs"),
            _ => panic!("MTGA reader does not support any other OS than windows and mac")
        };
        dirs.home_dir().join(os_dir).join(PARENT_DIR_1).join(PARENT_DIR_2).join(LOG_FILE_NAME)
    }
    else {
        panic!("Not supported")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_in_file() {
        read_file();
    }
}
