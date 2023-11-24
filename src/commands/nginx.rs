use clap::ArgMatches;
use run_script::run_script;
use std::{
    fs,
    process::{Command, Stdio},
};

use crate::{
    files::*,
    models::App,
    utils::{dirs::Dirs, networks::Network, spinner::Spinner},
};

pub struct Nginx;

impl Nginx {
    pub fn process_matches(args: &ArgMatches) {
        match args.subcommand() {
            Some(("run", args)) => {
                let build = args.get_flag("build");
                Nginx::run(build, false);
                Dirs::rm_temp();
            }
            Some(("stop", args)) => {
                let remove = args.get_flag("remove");
                Nginx::stop(remove, false);
            }
            _ => unreachable!(),
        }
    }

    pub fn run(build: bool, silent: bool) {
        let (_, output, _) = run_script!("docker ps -a | grep nbot_nginx").unwrap();
        if !output.is_empty() {
            run_script!("docker start nbot_nginx").unwrap();
            return;
        }

        // prepare files
        let temp_dir = Dirs::temp();
        let dockerfile = format!("{}/nbotnginx.Dockerfile", temp_dir);
        let entrypoint = format!("{}/nbotnginx_entrypoint.sh", temp_dir);
        let scheduler = format!("{}/nbotnginx_scheduler.txt", temp_dir);
        let default_conf = format!("{}/nbotnginx_default.conf", temp_dir);

        // create files
        fs::write(dockerfile, NGINX_DOCKERFILE).unwrap();
        fs::write(entrypoint, NGINX_ENTRYPOINT).unwrap();
        fs::write(
            scheduler,
            "0 12 * * * /usr/bin/certbot renew --quiet >> /var/log/cron.log 2>&1",
        )
        .unwrap();
        fs::write(default_conf, NGINX_DEFAULT_CONF).unwrap();

        let Ok((_, img, _)) = run_script!("docker images | grep nbot/nginx") else {
            return;
        };

        let exists = !img.is_empty();
        if !exists || build {
            if exists {
                let Ok(_) = run_script!("docker rm -f nbot_nginx; docker rmi nbot/nginx") else {
                    return;
                };
            }
            let dockerfile = format!("{}/nbotnginx.Dockerfile", temp_dir);
            let Ok(mut command) = Command::new("docker")
                .args([
                    "build",
                    "-t",
                    "nbot/nginx",
                    "-f",
                    dockerfile.as_str(),
                    temp_dir.as_str(),
                ])
                .stdout(if silent {
                    Stdio::null()
                } else {
                    Stdio::piped()
                })
                .spawn()
            else {
                return;
            };

            let Ok(_) = command.wait() else {
                return;
            };
        }

        let volume_dir = Dirs::nginx_volumes();

        Dirs::init_volumes();
        let docker_run = NGINX_RUN.replace("{{volume_dir}}", &volume_dir);

        let Ok((code, _, error)) = run_script!(docker_run) else {
            return;
        };

        if code != 0 {
            eprintln!("{error}");
        };
    }

    pub fn stop(remove: bool, silent: bool) {
        let mut spinner = Spinner::new();
        if !silent {
            spinner.start("Stopping Nginx ".to_owned());
        }

        run_script!("docker stop nbot_nginx").unwrap();
        if remove {
            run_script!("docker rm nbot_nginx").unwrap();
        }

        if !silent {
            spinner.stop("done".to_owned());
        }
    }

    pub fn connect_to_network(network: &Network) {
        match network {
            Network::Internal(_) => {}
            Network::Nginx(name) => {
                run_script!(format!("docker network connect {name} nbot_nginx"))
                    .unwrap_or_default();
            }
        };
    }

    pub fn disconnect_from_network(network: &Network) {
        match network {
            Network::Internal(_) => {}
            Network::Nginx(name) => {
                run_script!(format!("docker network disconnect {name} nbot_nginx"))
                    .unwrap_or_default();
            }
        };
    }

    pub fn is_running() -> bool {
        let (_, output, _) = run_script!("docker ps -a | grep nbot/nginx").unwrap();
        !output.is_empty()
    }

    pub fn add_conf(app: &App) {
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
        let domains = app.domains.as_ref().unwrap();
        let confd = Dirs::nginx_confd();
        for domain in domains {
            let file_path = format!("{}/{}.conf", confd, domain);
            if fs::read_to_string(&file_path).is_ok() {
                fs::remove_file(&file_path).unwrap();
            }
        }
    }

    pub fn generate_certificates(app: &App) {
        let domains = app.domains.as_ref().unwrap();

        for domain in domains {
            let certs_dir = Dirs::nginx_certs();
            let command = format!("mkdir -p {certs_dir}/live/{domain}");
            let email = app.email.as_ref().unwrap();

            run_script!(command).unwrap_or_default();

            let use_openssl = app.openssl.unwrap_or(false);

            // check if certificate exists
            let command = if use_openssl {
                format!(
                    r#"
                    docker exec nbot_nginx \
                    openssl x509 -checkend 86400 -noout \
                    -in /etc/letsencrypt/live/{domain}/fullchain.pem
                "#
                )
            } else {
                format!(
                    r#"
                    docker exec nbot_nginx \
                    certbot certificates | grep {domain}
                "#
                )
            };

            let (code, _, _) = run_script!(command).unwrap_or_default();

            if code == 0 {
                continue;
            }

            let command = if use_openssl {
                format!(
                    r#"
                    docker exec nbot_nginx \
                    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
                    -keyout /etc/letsencrypt/live/{domain}/privkey.pem \
                    -out /etc/letsencrypt/live/{domain}/fullchain.pem \
                    -subj "/C=""/ST=""/L=""/O=""/OU=""/CN={domain}"
                "#
                )
            } else {
                format!(
                    r#"
                    docker exec nbot_nginx \
                    certbot certonly --webroot -v \
                    -w /usr/share/nginx/html \
                    -d {domain} \
                    --email {email} \
                    --agree-tos \
                    --non-interactive
                "#
                )
            };

            let Ok((code, _, error)) = run_script!(command) else {
                continue;
            };
            if code != 0 {
                eprintln!("{error}");
            };
        }
    }
}

