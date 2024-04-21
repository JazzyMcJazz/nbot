use clap::ArgMatches;

use crate::{
    docker,
    models::App,
    utils::{contants::NGINX_CONTAINER_NAME, networks::Network},
    APP_STATE,
};

pub struct Nginx;

impl Nginx {
    pub async fn process_matches(args: &ArgMatches) {
        match args.subcommand() {
            Some(("run", args)) => {
                let build = args.get_flag("build");
                Nginx::run(build).await;
            }
            Some(("stop", args)) => {
                let remove = args.get_flag("remove");
                Nginx::stop(remove).await;
            }
            _ => unreachable!(),
        }
    }

    pub async fn run(build: bool) {
        let started = docker::containers::start_nginx().await;
        if started && !build {
            return;
        }

        let image = docker::images::find_by_name("nbot/nginx", Some("latest")).await;
        if image.is_none() || build {
            if let Some(image) = image {
                docker::images::remove(image.id.as_str()).await;
            }

            docker::images::build_nginx().await;
        }

        docker::containers::run_nginx().await;

        // find networks and connect to them
        let projects = APP_STATE.to_owned().projects;
        'project_loop: for project in projects {
            for app in project.apps {
                if app.domains.is_some() {
                    let network = Network::nginx_from_project(&project.name);
                    Nginx::connect_to_network(&network).await;
                    continue 'project_loop;
                }
            }
        }
    }

    pub async fn stop(remove: bool) {
        let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let container = docker::containers::find_by_name(&name).await;
        let Some(container) = container else {
            return;
        };
        let Some(id) = container.id else {
            return;
        };

        docker::containers::stop(id.as_str()).await;
        if remove {
            docker::containers::remove(id.as_str()).await;
        }
    }

    pub async fn connect_to_network(network: &Network) {
        match network {
            Network::Internal(_) => {}
            Network::Nginx(_) => {
                let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
                let container = docker::containers::find_by_name(&name).await;
                let Some(container) = container else {
                    eprintln!("Nginx container not found");
                    return;
                };

                let container_id = container.id.unwrap();
                let is_connected =
                    docker::network::is_connected(container_id.as_str(), network).await;
                if !is_connected {
                    docker::network::connect(container_id.as_str(), network).await;
                }
            }
        };
    }

    pub async fn disconnect_from_network(network: &Network) {
        match network {
            Network::Internal(_) => {}
            Network::Nginx(_) => {
                let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
                let container = docker::containers::find_by_name(&name).await;
                let Some(container) = container else {
                    return;
                };

                docker::network::disconnect(container, network).await;
            }
        };
    }

    pub async fn is_running() -> bool {
        let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let container = docker::containers::find_by_name(&name).await;
        let Some(container) = container else {
            return false;
        };

        if let Some(state) = &container.state {
            return state == "running";
        }

        false
    }

    pub async fn add_conf(app: &App) {
        if app.domains.is_none() {
            return;
        }

        let domains = app
            .domains
            .as_ref()
            .expect("domains not found when adding conf");
        if domains.is_empty() {
            return;
        }

        let file_name = &domains.first().unwrap();
        let container_name = &app.container_name;
        let nginx_container = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let port = match &app.port {
            Some(port) => port,
            None => "80",
        };

        let mut cmd = vec![
            "sh",
            "/functions.sh",
            "add_conf",
            file_name.as_str(),
            container_name,
            port,
        ];
        cmd.extend(domains.iter().map(|d| d.as_str()));

        docker::exec::exec(&nginx_container, &cmd).await;
    }

    pub async fn remove_conf(app: &App) {
        if app.domains.is_none() {
            return;
        }

        let domains = app.domains.as_ref().unwrap();
        if domains.is_empty() {
            return;
        }

        let file_name = &domains.first().unwrap();
        let cmd = vec!["sh", "/functions.sh", "remove_conf", file_name.as_str()];
        let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let (_, code, error) = docker::exec::exec(&name, &cmd).await;

        if code != 0 {
            eprintln!("Error removing conf");
            eprintln!("{}", error);
        }
    }

    pub async fn generate_certificates(app: &App) {
        if app.domains.is_none() {
            return;
        }
        let domains = app.domains.as_ref().unwrap();
        let name = format!("{}{}", APP_STATE.container_prefix, NGINX_CONTAINER_NAME);
        let container = docker::containers::find_by_name(&name).await;
        let Some(container) = container else {
            return;
        };
        let Some(container_id) = container.id else {
            return;
        };

        let use_openssl = app.openssl.unwrap_or(false);

        let cmd = if use_openssl {
            vec![
                "sh",
                "/functions.sh",
                "generate_certs_openssl",
                domains.first().unwrap(),
            ]
        } else {
            let email = app.email.as_ref().unwrap();
            let mut cmd = vec!["sh", "/functions.sh", "generate_certs_certbot", email];
            cmd.extend(domains.iter().map(|d| d.as_str()));
            cmd
        };

        let (_, code, error) = docker::exec::exec(container_id.as_str(), &cmd).await;
        if code != 0 {
            eprintln!("Error generating certificate");
            eprintln!("{}", error);
        }
    }
}
