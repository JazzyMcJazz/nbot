use std::thread::sleep;

use crate::{models::Project, utils::networks::Network, APP_STATE};
use run_script::run_script;

use super::nginx::Nginx;

pub struct Run;

impl Run {
    pub fn project(project: Project) {
        if !Nginx::is_running() {
            Nginx::run(false, true);
        }

        let networks = (
            Network::internal_from_project(&project.name).create(),
            Network::nginx_from_project(&project.name).create(),
        );

        for app in &project.apps {
            app.run(&vec![&networks.0, &networks.1]);
            let mut up = false;

            if app.domains.is_some() {
                // wait until container is up
                for seconds in [1, 3, 5] {
                    let command = if let Some(port) = &app.port {
                        format!(
                            "docker exec nbot_nginx curl -Iks http://{}:{}",
                            &app.container_name, port
                        )
                    } else {
                        format!(
                            "docker exec nbot_nginx curl -Iks http://{}",
                            &app.container_name
                        )
                    };

                    let (code, _, _) = run_script!(command).unwrap_or_default();

                    dbg!(code);

                    if code != 0 {
                        if seconds != 5 {
                            sleep(std::time::Duration::from_secs(seconds));
                        }
                        continue;
                    }

                    up = true;
                    break;
                }
            }

            if !up {
                app.stop();
                eprintln!("Failed to start app {}", app.name);
                continue;
            }

            Nginx::generate_certificates(app, true);
            Nginx::add_conf(app);
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
