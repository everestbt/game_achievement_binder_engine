use directories::{ProjectDirs};
use std::{fs, path::PathBuf};

pub fn get_local_dir(filename: &str) -> PathBuf {
    let binding = ProjectDirs::from("com", "everest", "steam_randomiser")
        .expect("Failed to get project directories");
    let data_dir =  binding.data_local_dir();
    if !fs::exists(data_dir).expect("Failed to check for directory") {
        fs::create_dir(data_dir).expect("Failed to create directory");
    }
    data_dir.join(filename)
}