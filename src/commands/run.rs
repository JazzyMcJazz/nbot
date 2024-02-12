use std::{io::Write, process, thread::sleep};

use crate::{
    models::Project,
    utils::{networks::Network, spinner::Spinner},
    APP_STATE,
};
use run_script::run_script;

use super::nginx::Nginx;

pub struct Run;

impl Run {
    pub fn project(project: Project, force: bool) {
        let mut app_state = APP_STATE.to_owned();
        if !force && app_state.exists(&project.name) {
            let mut line = String::new();
            print!("Project already exists. Override? (y/n): ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut line).unwrap();
            if line.trim() != "y" {
                process::exit(1);
            }
        }

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
            let mut reason = String::new();

            if app.domains.is_some() {
                // wait until container is up
                for seconds in 1..4 {
                    if seconds != 4 {
                        sleep(std::time::Duration::from_secs(seconds));
                    } else {
                        break;
                    }
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

                    let (code, output, error) = run_script!(command).unwrap_or_default();
                    reason = if error.is_empty() {
                        output
                    } else {
                        error
                    };

                    if code == 0 {
                        up = true;
                        break;
                    }
                }
                Nginx::generate_certificates(app);
                Nginx::add_conf(app);
            } else {
                // check if container is up
                for seconds in 1..3 {
                    let (code, _, error) =
                        run_script!(format!("docker ps -q -f name={}", app.container_name))
                            .unwrap_or_default();
                    reason = error;

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
                spinner.stop(format!("{}: failed. Reason: {}", app.name, reason.trim()));
                continue;
            } else {
                spinner.stop(format!("{}: started", app.name));
            }
        }

        app_state.add_or_update_project(project);
    }
}
