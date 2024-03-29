use clap::ArgMatches;

use crate::{utils::networks::Network, APP_STATE};

use super::nginx::Nginx;

pub struct Rm;

impl Rm {
    pub async fn projects(args: &ArgMatches) {
        let mut state = APP_STATE.clone();

        let projects: Vec<String> = args
            .get_many("project")
            .unwrap_or_default()
            .cloned()
            .collect();

        let mut projects_to_remove = vec![];
        let mut projects_to_keep = vec![];

        for project in state.projects {
            if projects.contains(&project.name) {
                projects_to_remove.push(project);
            } else {
                projects_to_keep.push(project);
            }
        }

        for project in projects_to_remove {
            for app in &project.apps {
                Nginx::remove_conf(app).await;
                if app.is_running().await {
                    app.stop().await;
                }
                app.remove().await;
            }

            let project_net = Network::internal_from_project(&project.name);
            let nginx_net = Network::nginx_from_project(&project.name);
            Nginx::disconnect_from_network(&nginx_net).await;
            project_net.remove().await;
            nginx_net.remove().await;
        }

        state.projects = projects_to_keep;
        state.save();
    }
}
