use std::fs;
pub struct Dirs;

impl Dirs {
    fn dir() -> &'static str {
        let config_dir = "/etc/nbot";

        match fs::read_dir(config_dir) {
            Ok(_) => {}
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {}
                    _ => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }
                match fs::create_dir_all(config_dir) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }
            }
        }

        config_dir
    }

    pub fn _config_dir() -> &'static str {
        Self::dir()
    }

    pub fn config_file() -> String {
        let config_dir = Self::dir();
        format!("{}/config.json", config_dir)
    }

    pub fn rm_all() {
        let config_dir = Self::dir();
        let entries = match fs::read_dir(config_dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        };

        for entry in entries {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };

            if path.is_dir() {
                match fs::remove_dir_all(&path) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }
            } else {
                match fs::remove_file(&path) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                };
            }
        }
    }
}
