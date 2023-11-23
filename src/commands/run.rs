use crate::{models::Project, utils::networks::Network, APP_STATE};

use super::nginx::Nginx;

pub struct Run;

impl Run {
    pub fn project(project: Project) {
        if !Nginx::is_running() {
            Nginx::run(false, true);
        }

        let networks = (
            Network::Internal(format!("nbot_{}_net", &project.name)).create(),
            Network::Nginx(format!("nbot_nginx_{}_net", &project.name)).create(),
        );

        for app in &project.apps {
            app.run(&vec![&networks.0, &networks.1]);
            if app.domains.is_some() {
                Nginx::add_conf(app)
            }
        }

        Nginx::connect_to_network(&networks.1);

        let mut app_state = APP_STATE.to_owned();
        app_state.add_or_update_project(project);
    }
}

// --env MARIADB_USER=example-user
// --env MARIADB_PASSWORD=my_cool_secret
// --env MARIADB_DATABASE=exmple-database
// --env MARIADB_ROOT_PASSWORD=my-secret-pw
// mariadb:latest
