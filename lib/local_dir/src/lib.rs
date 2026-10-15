use directories::{ProjectDirs};
use std::{
    env, 
    fs, 
    path::PathBuf,
};

pub fn get_local_dir(filename: &str) -> PathBuf {
    // Get a database suffix if set, used for testing
    let dir_suffix = env::var("GABE_DIR_SUFFIX").ok();
    let dir_name = "steam_randomiser".to_owned() + &dir_suffix.unwrap_or("".to_string());
    
    let binding = ProjectDirs::from("com", "everest", &dir_name)
        .expect("Failed to get project directories");
    let data_dir =  binding.data_local_dir();
    if !fs::exists(data_dir).expect("Failed to check for directory") {
        fs::create_dir(data_dir).expect("Failed to create directory");
    }
    data_dir.join(filename)
}