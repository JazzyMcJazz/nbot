use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, io::Write, process};

use crate::{models::Project, utils::dirs::Dirs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppState {
    pub container_prefix: String,
    pub image_prefix: String,
    pub network_prefix: String,
    pub projects: Vec<Project>,
}

impl AppState {
    fn default() -> Self {
        Self {
            container_prefix: String::from("nbot_"),
            image_prefix: String::from("nbot_"),
            network_prefix: String::from("nbot_"),
            projects: vec![],
        }
    }
    pub fn from_storage() -> Self {
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

    pub fn add_or_update_project(&mut self, project: &Project) {
        let project = project.clone();
        if let Some(existing_project) = self.projects.iter_mut().find(|p| p.name == project.name) {
            existing_project.apps = project.apps;
        } else {
            self.projects.push(project);
        }
        self.save();
    }

    pub fn save(&self) {
        let config_file = Dirs::config_file();
        let config = serde_json::to_string(&self).unwrap();
        match fs::write(config_file, config) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }

    pub fn exists(&self, project_name: &str) -> bool {
        self.projects.iter().any(|p| p.name == project_name)
    }
}
