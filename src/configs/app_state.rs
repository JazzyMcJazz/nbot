use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, io::Write, process};

use crate::utils::dirs::Dirs;

use super::project::Project;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppState {
    projects: Vec<Project>,
}

impl AppState {
    fn default() -> Self {
        Self { projects: vec![] }
    }
    pub fn new() -> Self {
        let config_file = Dirs::config_file();

        if let Ok(config) = fs::read_to_string(config_file) {
            match serde_json::from_str(&config) {
                Ok(state) => state,
                Err(_) => {
                    let mut line = String::new();
                    print!("Config file is invalid. Override? This deletes everything!! (y/n): ");
                    std::io::stdout().flush().unwrap();
                    std::io::stdin().read_line(&mut line).unwrap();
                    if line.trim() == "y" {
                        let state = Self::default();
                        state.save();
                        state
                    } else {
                        process::exit(1);
                    }
                }
            }
        } else {
            let state = Self::default();
            state.save();
            state
        }
    }
    pub fn save(&self) {
        let config_file = Dirs::config_file();
        let config = serde_json::to_string(&self).unwrap();
        fs::write(config_file, config).expect("Failed to write config file");
    }
}
