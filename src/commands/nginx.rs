use clap::ArgMatches;
use run_script::run_script;
use std::fs;

use crate::{
    docker,
    files::*,
    models::App,
    utils::{dirs::Dirs, networks::Network},
};

pub struct Nginx;

impl Nginx {
    pub async fn process_matches(args: &ArgMatches) {
        match args.subcommand() {
            Some(("run", args)) => {
                let build = args.get_flag("build");
                Nginx::run(build).await;
                Dirs::rm_temp();
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
        if started {
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
    }

    pub async fn stop(remove: bool) {
        let container = docker::containers::find_by_name("nbot_nginx").await;
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
                let container = docker::containers::find_by_name("nbot_nginx").await;
                let Some(container) = container else {
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
                let container = docker::containers::find_by_name("nbot_nginx").await;
                let Some(container) = container else {
                    return;
                };

                docker::network::disconnect(container, network).await;
            }
        };
    }

    pub async fn is_running() -> bool {
        let container = docker::containers::find_by_name("nbot_nginx").await;
        let Some(container) = container else {
            return false;
        };

        if let Some(state) = &container.state {
            return state == "running";
        }

        false
    }

    pub fn add_conf(app: &App) {
        if app.domains.is_none() {
            return;
        }

        let domains = app.domains.as_ref().unwrap();
        let mut files = Vec::<(String, String)>::new();
        for domain in domains {
            let mut conf = NGINX_TEMPLATE_CONF
                .replace("{{name}}", &app.container_name)
                .replace("{{domain}}", domain);

            let port = match &app.port {
                Some(port) => port,
                None => "80",
            };

            conf = conf.replace("{{port}}", port);

            // If domain does not have a subdomain, add www subdomain
            let www_domain = if domain.split('.').count() == 2 {
                format!(" www.{}", domain) // must have space in front
            } else {
                "".to_owned()
            };
            conf = conf.replace("{{www_domain}}", www_domain.as_str());

            files.push((domain.to_owned(), conf));
        }

        let confd = Dirs::nginx_confd();
        for (file_name, content) in files {
            let file_path = format!("{}/{}.conf", confd, file_name);
            // if fs::read_to_string(&file_path).is_err() {
            fs::write(&file_path, content).unwrap();
            // }
        }
    }

    pub fn remove_conf(app: &App) {
        if app.domains.is_none() {
            return;
        }
        let domains = app.domains.as_ref().unwrap();
        let confd = Dirs::nginx_confd();
        for domain in domains {
            let file_path = format!("{}/{}.conf", confd, domain);
            if fs::read_to_string(&file_path).is_ok() {
                fs::remove_file(&file_path).unwrap();
            }
        }
    }

    pub async fn generate_certificates(app: &App) {
        if app.domains.is_none() {
            return;
        }
        let domains = app.domains.as_ref().unwrap();
        let container = docker::containers::find_by_name("nbot_nginx").await;
        let Some(container) = container else {
            return;
        };
        let Some(container_id) = container.id else {
            return;
        };

        for domain in domains {
            let certs_dir = Dirs::nginx_certs();
            let command = format!("mkdir -p {certs_dir}/live/{domain}");
            let email = app.email.as_ref().unwrap();

            run_script!(command).unwrap_or_default();

            let use_openssl = app.openssl.unwrap_or(false);

            // check if certificate exists
            let file_path = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain);
            let command = if use_openssl {
                vec![
                    "openssl",
                    "x509",
                    "-checkend",
                    "86400",
                    "-noout",
                    "-in",
                    file_path.as_str(),
                ]
            } else {
                vec!["certbot", "certificates", "|", "grep", domain]
            };

            // if certificate exists, continue
            let (output, code, ..) = docker::exec::exec(container_id.as_str(), &command).await;
            if use_openssl && code == 0 {
                continue;
            }

            if !use_openssl && !output.is_empty() {
                continue;
            }

            let file_path = format!("/etc/letsencrypt/live/{domain}/privkey.pem");
            let subject = format!("/C=''/ST=''/L=''/O=''/OU=''/CN={domain}");
            let output = format!("/etc/letsencrypt/live/{domain}/fullchain.pem");
            let command = if use_openssl {
                vec![
                    "openssl",
                    "req",
                    "-x509",
                    "-nodes",
                    "-days",
                    "365",
                    "-newkey",
                    "rsa:2048",
                    "-keyout",
                    file_path.as_str(),
                    "-out",
                    output.as_str(),
                    "-subj",
                    subject.as_str(),
                ]
            } else {
                vec![
                    "certbot",
                    "certonly",
                    "--webroot",
                    "-v",
                    "-w",
                    "/usr/share/nginx/html",
                    "-d",
                    domain,
                    "--email",
                    email,
                    "--agree-tos",
                    "--non-interactive",
                ]
            };

            let (_, code, error) = docker::exec::exec(container_id.as_str(), &command).await;
            if code != 0 {
                eprintln!("Error generating certificate for {}", domain);
                eprintln!("{}", error);
            }
        }
    }
}
