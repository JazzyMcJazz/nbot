use std::thread::sleep;

use crate::{
    models::Project,
    utils::{networks::Network, spinner::Spinner},
    APP_STATE,
};
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

        Nginx::connect_to_network(&networks.1);

        let mut spinner = Spinner::new();
        for app in &project.apps {
            spinner.start(format!("{}: ", app.name));

            app.run(&vec![&networks.0, &networks.1]);
            let mut up = false;

            if app.domains.is_some() {
                // wait until container is up
                for seconds in 1..3 {
                    let command = if let Some(port) = &app.port {
                        format!(
                            "docker exec nbot_nginx curl -Is http://{}:{}",
                            &app.container_name, port
                        )
                    } else {
                        format!(
                            "docker exec nbot_nginx curl -Is http://{}",
                            &app.container_name
                        )
                    };

                    let (code, _, _) = run_script!(command).unwrap_or_default();

                    if code == 0 {
                        up = true;
                        break;
                    }

                    if seconds != 3 {
                        sleep(std::time::Duration::from_secs(seconds));
                    }
                }
            }

            if !up {
                app.stop();
                spinner.stop(format!("{}: failed", app.name));
                continue;
            } else {
                spinner.stop(format!("{}: OK", app.name));
            }

            Nginx::generate_certificates(app);
            Nginx::add_conf(app);
        }

        let mut app_state = APP_STATE.to_owned();
        app_state.add_or_update_project(project);
    }
}
