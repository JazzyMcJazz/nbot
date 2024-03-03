use std::vec;
use tabled::{Table, Tabled};

use crate::{docker, models::App, utils::contants::NGINX_CONTAINER_NAME, APP_STATE};

#[derive(Tabled)]
struct AppStatus {
    project: String,
    service: String,
    container_name: String,
    port: String,
    status: String,
    domains: String,
    image: String,
    certificate: String,
}

impl AppStatus {
    pub async fn from_apps(apps: Vec<App>, project: &String) -> Vec<Self> {
        let mut app_statuses = vec![];
        for app in apps {
            let (_, status) = Self::get_app_status(&app).await;

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

            let certificate = match app.openssl {
                Some(openssl) => if openssl { "openssl" } else { "letsencrypt" }.to_owned(),
                None => String::new(),
            };

            let port = match app.port {
                Some(port) => port.to_string(),
                None => "".to_owned(),
            };

            app_statuses.push(AppStatus {
                project: project.to_owned(),
                service: app.name,
                container_name: app.container_name,
                port,
                status,
                domains,
                image: app.image,
                certificate,
            });
        }
        app_statuses
    }

    async fn get_app_status(app: &App) -> (String, String) {
        let container = docker::containers::find_by_name(&app.container_name).await;
        let Some(container) = container else {
            return (String::new(), "container not found".to_owned());
        };

        let mut container_id = container.id.unwrap();
        let status = container.state.unwrap();

        container_id.truncate(12);
        container_id += "...";

        (container_id, status)
    }
}

pub struct Status {
    nginx: String,
    apps: Vec<AppStatus>,
}

impl Status {
    pub async fn new() -> Self {
        let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let container = docker::containers::find_by_name(&name).await;
        let mut nginx_status = "Nginx: not found".to_owned();
        if let Some(container) = container {
            let status = container.state.unwrap();
            nginx_status = format!("Nginx: {}", status);
        }

        let state = APP_STATE.clone();
        let mut apps = vec![];
        for project in state.projects {
            apps.append(&mut AppStatus::from_apps(project.apps, &project.name).await);
        }

        Status {
            nginx: nginx_status,
            apps,
        }
    }

    pub fn display(&self) {
        println!("\n{}\n", self.nginx);
        let table = Table::new(&self.apps);
        if !self.apps.is_empty() {
            println!("{}", table);
        }
    }
}
