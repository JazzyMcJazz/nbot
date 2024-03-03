use std::fs;

use directories::ProjectDirs;

pub struct Dirs;

impl Dirs {
    fn dir() -> String {
        let dirs = ProjectDirs::from("dev", "treeleaf", "nbot");

        let Some(dirs) = dirs else {
            eprintln!("Failed to get config directory");
            std::process::exit(1);
        };

        let config_dir = dirs.config_dir();
        if fs::read_dir(config_dir).is_err() {
            if fs::create_dir_all(config_dir).is_err() {
                eprintln!("Failed to create config directory");
                std::process::exit(1);
            }
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
        if fs::remove_dir_all(config_dir).is_err() {
            eprintln!("Failed to remove config directory");
            std::process::exit(1);
        }
    }
}
