use serde::{Deserialize, Serialize};
use serde_json;
use std::{fs, io::Write, process};

use crate::{models::Project, utils::dirs::Dirs};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppState {
    container_prefix: String,
    image_prefix: String,
    network_prefix: String,
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

    pub fn add_or_update_project(&mut self, project: Project) {
        if let Some(existing_project) = self.projects.iter_mut().find(|p| p.name == project.name) {
            existing_project.apps = project.apps;

            // for app in project.apps {
            //     if let Some(existing_app) = existing_project
            //         .apps
            //         .iter_mut()
            //         .find(|a| a.name == app.name)
            //     {
            //         existing_app.update(app);
            //     } else {
            //         existing_project.apps.push(app.to_owned());
            //     }
            // }
        } else {
            self.projects.push(project);
        }
        self.save();
    }

    pub fn save(&self) {
        let config_file = Dirs::config_file();
        let config = serde_json::to_string(&self).unwrap();
        fs::write(config_file, config).expect("Failed to write config file");
    }

    pub fn exists(&self, project_name: &str) -> bool {
        self.projects.iter().any(|p| p.name == project_name)
    }
}
