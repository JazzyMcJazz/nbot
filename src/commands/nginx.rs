use clap::ArgMatches;
use run_script::run_script;
use std::{
    fs,
    process::{Command, Stdio},
};

use crate::{
    files::*,
    utils::{dirs::Dirs, spinner::Spinner},
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

        // create files
        fs::write(dockerfile, NGINX_DOCKERFILE).unwrap();
        fs::write(entrypoint, NGINX_ENTRYPOINT).unwrap();
        fs::write(
            scheduler,
            "0 12 * * * /usr/bin/certbot renew --quiet >> /var/log/cron.log 2>&1",
        )
        .unwrap();

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
        fs::create_dir_all(&volume_dir).unwrap();

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
            spinner.start("Stopping Nginx ");
        }

        run_script!("docker stop nbot_nginx").unwrap();
        if remove {
            run_script!("docker rm nbot_nginx").unwrap();
        }

        if !silent {
            spinner.stop("done");
        }
    }
}
