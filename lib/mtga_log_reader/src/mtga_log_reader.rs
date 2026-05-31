use directories::{BaseDirs};
use std::env::consts::OS;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use serde_json::Value;
use convert_case::ccase;

static PARENT_DIR_1: & str = "Wizards Of The Coast";
static PARENT_DIR_2: & str = "MTGA";
static LOG_FILE_NAME: & str = "Player.log";

struct Achievement {
    name: String,
    achieved: bool
}

fn read_file() {
    let path = get_path();
    if let Ok(file) = File::open(path) {
        let lines = BufReader::new(file).lines();

        let mut achievements = vec![];
        for l in lines.map_while(Result::ok) {
            if l.starts_with("{\"NodeStates\":{\"---META_WelcomeToArena\"") {
                let node = Value::from_str(l.as_str()).unwrap();
                let achievement_map = node.get("NodeStates").unwrap();
                for v in achievement_map.as_object().unwrap().iter() {
                    let name = format_achievement_name(v.0);
                    let achieved = v.1.get("Status").and_then(Value::as_str).map(|f| f == "Completed").unwrap_or(false);
                    achievements.push(Achievement {name, achieved});
                }
            }
        }
        for a in achievements {
            println!("{} : {}", a.name, a.achieved)
        }
    }
}

fn format_achievement_name(string: &str) -> String {
    ccase!(pascal -> title, string)
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
