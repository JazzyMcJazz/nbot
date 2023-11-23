use crate::{models::Project, utils::networks::Network, APP_STATE};

use super::nginx::Nginx;

pub struct Run;

impl Run {
    pub fn project(project: Project) {
        if !Nginx::is_running() {
            Nginx::run(false, true);
        }

        let internal_network = Network::Internal(format!("nbot_{}_net", &project.name)).create();
        let nginx_network = Network::Nginx(format!("nbot_nginx_{}_net", &project.name)).create();

        for app in &project.apps {
            app.run(&vec![&internal_network, &nginx_network]);
        }

        Nginx::connect_to_network(&nginx_network);

        let mut app_state = APP_STATE.to_owned();
        app_state.add_or_update_project(project);
    }
}

// --env MARIADB_USER=example-user
// --env MARIADB_PASSWORD=my_cool_secret
// --env MARIADB_DATABASE=exmple-database
// --env MARIADB_ROOT_PASSWORD=my-secret-pw
// mariadb:latest
