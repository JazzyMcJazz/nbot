use clap::ArgMatches;
use run_script::run_script;
use std::{
    fs,
    process::{Command, Stdio},
};

use crate::{files::*, utils::spinner::Spinner};

pub struct Nginx;

impl Nginx {
    pub fn process_args(args: &ArgMatches) {
        match args.subcommand() {
            Some(("run", args)) => {
                let build = args.get_flag("build");
                Nginx::run(build, false);
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
        let (_, output, _) =
            run_script!("getent passwd $(whoami) | awk -F ':' '{print $6}'").unwrap();
        let home = output.replace('\n', "");
        let dockerfile = format!("{}/nbotnginx.Dockerfile", home.trim());
        let entrypoint = format!("{}/nbotnginx_entrypoint.sh", home.trim());
        let scheduler = format!("{}/nbotnginx_scheduler.txt", home.trim());

        // create files
        fs::write(&dockerfile, NGINX_DOCKERFILE).unwrap();
        fs::write(&entrypoint, NGINX_ENTRYPOINT).unwrap();
        fs::write(
            &scheduler,
            "0 12 * * * /usr/bin/certbot renew --quiet >> /var/log/cron.log 2>&1",
        )
        .unwrap();

        let Ok((_, img, _)) = run_script!("docker images | grep nbot/nginx") else {
            Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
            return;
        };

        let exists = !img.is_empty();
        if !exists || build {
            if exists {
                let Ok(_) = run_script!("docker rm -f nbot_nginx; docker rmi nbot/nginx") else {
                    Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
                    return;
                };
            }
            let dockerfile = format!("{}/nbotnginx.Dockerfile", home.trim());
            let Ok(mut command) = Command::new("docker")
                .args([
                    "build",
                    "-t",
                    "nbot/nginx",
                    "-f",
                    dockerfile.as_str(),
                    home.trim(),
                ])
                .stdout(if silent {
                    Stdio::null()
                } else {
                    Stdio::piped()
                })
                .spawn()
            else {
                Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
                return;
            };

            let Ok(_) = command.wait() else {
                Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
                return;
            };
        }

        let volume_dir = format!("{}/nbotnginx", home.trim());
        fs::create_dir_all(&volume_dir).unwrap();

        let docker_run = NGINX_RUN.replace("{{volume_dir}}", &volume_dir);

        let Ok((code, _, error)) = run_script!(docker_run) else {
            Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
            return;
        };

        if code != 0 {
            Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
            eprintln!("{error}");
            return;
        }

        // Cleanup
        Nginx::cleanup(vec![&dockerfile, &entrypoint, &scheduler]);
        run_script!("rm $HOME/nbotnginx.Dockerfile $HOME/nbotnginx_entrypoint.sh $HOME/nbotnginx_scheduler.txt").unwrap();
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

    fn cleanup(files: Vec<&str>) {
        for file in files {
            fs::remove_file(file).unwrap();
        }
    }
}
