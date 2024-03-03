use std::{collections::HashSet, io::Write, process};

use crate::{docker, utils::dirs::Dirs, APP_STATE};

pub struct Reset;

impl Reset {
    pub async fn execute(force: bool) {
        if !force {
            let mut line = String::new();
            print!("Are you sure you want to reset? This will delete all projects, containers and volumes. (y/n): ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut line).unwrap();
            if line.trim() != "y" {
                process::exit(1);
            }
        }

        let projects = APP_STATE.projects.clone();

        let mut volumes = HashSet::new();
        let mut networks = HashSet::new();

        let nginx_container_name = format!("{}nginx", APP_STATE.container_prefix);
        volumes.extend(docker::volumes::find_by_container(&nginx_container_name).await);
        networks.extend(docker::network::find_ids_by_container(&nginx_container_name).await);

        for project in projects {
            for app in project.apps {
                let container = docker::containers::find_by_name(&app.container_name).await;
                if let Some(container) = container {
                    let Some(container_id) = container.id else {
                        continue;
                    };
                    let container_networks =
                        docker::network::find_ids_by_container(&container_id).await;
                    networks.extend(container_networks);
                }
            }
        }

        // Stop and remove nginx and all project containers
        super::UpDown::down().await;
        docker::network::remove_many(networks.into_iter().collect()).await;
        docker::volumes::remove_many(volumes.into_iter().collect(), true).await;
        Dirs::rm_all();
    }
}
