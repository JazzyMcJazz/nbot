use run_script::run_script;
use std::vec;
use tabled::{Table, Tabled};

use crate::{models::App, APP_STATE};

#[derive(Tabled)]
struct AppStatus {
    project: String,
    service: String,
    container_id: String,
    container_name: String,
    status: String,
    ports: String,
    domains: String,
    image: String,
    volumes: String,
    env_vars: String,
    certificate: String,
}

impl AppStatus {
    pub fn from_apps(apps: Vec<App>, project: &String) -> Vec<Self> {
        let mut app_statuses = vec![];
        for app in apps {
            let (container_id, status) = Self::get_app_status(&app);
            let ports = match app.port {
                Some(port) => port,
                None => String::new(),
            };
            let domains = match app.domains {
                Some(domains) => {
                    let mut domains_str = String::new();
                    for domain in domains {
                        let linebreak = if domains_str.is_empty() { "" } else { "\n" };
                        domains_str.push_str(&format!("{linebreak}{domain}"));
                    }
                    domains_str
                }
                None => String::new(),
            };

            let mut env_vars = String::new();
            for env_var in app.env_vars {
                let linebreak = if env_vars.is_empty() { "" } else { "\n" };
                let var = env_var.split('=').collect::<Vec<&str>>()[0];
                env_vars.push_str(&format!("{linebreak}{var}=***"));
            }

            let mut volumes = String::new();
            for volume in app.volumes {
                let linebreak = if volumes.is_empty() { "" } else { "\n" };
                volumes.push_str(&format!("{linebreak}{volume}"));
            }

            let certificate = match app.openssl {
                Some(openssl) => if openssl { "openssl" } else { "letsencrypt" }.to_owned(),
                None => String::new(),
            };

            app_statuses.push(AppStatus {
                project: project.to_owned(),
                service: app.name,
                container_id,
                container_name: app.container_name,
                status,
                ports,
                domains,
                image: app.image,
                env_vars,
                volumes,
                certificate,
            });
        }
        app_statuses
    }

    fn get_app_status(app: &App) -> (String, String) {
        let (_, output, _) =
            run_script!(format!("docker ps -a -q -f name={}", app.container_name)).unwrap();

        let container_id = output.replace('\n', "");

        let (_, output, _) = run_script!(format!(
            "docker inspect --format='{{{{.State.Status}}}}' {}",
            container_id
        ))
        .unwrap();

        let mut status = output.replace('\n', "");
        if status.is_empty() {
            status = String::from("missing");
        }
        (container_id, status)
    }
}

pub struct Status {
    apps: Vec<AppStatus>,
}

impl Status {
    pub fn new() -> Self {
        let state = APP_STATE.clone();
        let mut apps = vec![];
        for project in state.projects {
            apps.append(&mut AppStatus::from_apps(project.apps, &project.name));
        }

        Status { apps }
    }

    pub fn display(&self) {
        let table = Table::new(&self.apps);
        println!("{}", table);
    }
}
