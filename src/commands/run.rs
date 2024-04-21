use std::{io::Write, process, thread::sleep};

use crate::{
    docker,
    models::{App, Project},
    utils::networks::Network,
    APP_STATE,
};

use super::nginx::Nginx;

pub struct Run;

impl Run {
    pub async fn project(project: Project, force: bool) {
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

        // Update app_state and save
        app_state.add_or_update_project(&project);

        if !Nginx::is_running().await {
            Nginx::run(false).await;
        }

        let networks = (
            Network::internal_from_project(&project.name).create().await,
            Network::nginx_from_project(&project.name).create().await,
        );

        Nginx::connect_to_network(&networks.1).await;

        let apps = App::topological_sort_by_dependenceis(&project.apps);
        for app in &apps {
            let started = app.run(&vec![&networks.0, &networks.1]).await;
            sleep(std::time::Duration::from_secs(1));

            if !started {
                println!("{}: failed", app.name);
                continue;
            }

            let mut up = false;
            let mut reason = String::new();

            if app.domains.is_some() {
                // wait until container is up
                for seconds in 1..15 {
                    // pinging a container immediately after starting it
                    // has been known to cause it to crash. Therefore, we
                    // wait at the start of the loop instead of the end.
                    sleep(std::time::Duration::from_secs(seconds));

                    if !app.is_running().await {
                        app.start().await;
                    }

                    let url = if let Some(port) = &app.port {
                        format!("http://{}:{}", &app.container_name, port)
                    } else {
                        format!("http://{}", &app.container_name)
                    };
                    let cmd = vec!["curl", "-I", url.as_str()];
                    // format!("curl -I http://{}", &app.container_name)

                    let name = format!("{}nginx", app_state.container_prefix);
                    let (output, code, error) = docker::exec::exec(&name, &cmd).await;

                    // TODO: check result

                    reason = if error.is_empty() { output } else { error };

                    if code == 0 {
                        up = true;
                        break;
                    }
                }
                Nginx::generate_certificates(app).await;
                Nginx::add_conf(app).await;
            } else {
                // check if container is up
                for seconds in 1..3 {
                    sleep(std::time::Duration::from_secs(seconds));

                    let container = docker::containers::find_by_name(&app.container_name).await;
                    if let Some(container) = container {
                        if let Some(state) = &container.state {
                            if state == "running" {
                                up = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !up {
                eprintln!("{}: failed. Reason: {}", app.name, reason);
                println!("Note: If the service takes a long time to spin up, it may not in fact be failing. Run nbot status to check the status of the container.");
                continue;
            }
        }
    }
}
