use std::fs;

use directories::ProjectDirs;

pub struct Dirs;

impl Dirs {
    fn dir() -> String {
        let dirs = ProjectDirs::from("dev", "treeleaf", "nbot").unwrap();
        let config_dir = dirs.config_dir();
        if fs::read_dir(config_dir).is_err() {
            fs::create_dir_all(config_dir).expect("Failed to create config directory");
        }
        config_dir
            .to_str()
            .expect("Failed to convert config directory to string")
            .to_owned()
    }

    pub fn _config_dir() -> String {
        Self::dir()
    }

    pub fn config_file() -> String {
        let config_dir = Self::dir();
        format!("{}/config.json", config_dir)
    }

    pub fn rm_all() {
        let config_dir = Self::dir();
        fs::remove_dir_all(config_dir).expect("Failed to remove config directory");
    }
}
