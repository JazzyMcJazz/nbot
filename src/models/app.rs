use std::process::{Command, Stdio};

use clap::ArgMatches;
use run_script::run_script;
use serde::{Deserialize, Serialize};

use crate::utils::networks::Network;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    pub name: String,
    image: String,
    pub container_name: String,
    pub ports: Vec<String>,
    env_vars: Vec<String>,
    volumes: Vec<String>,
    depends_on: Vec<String>,
    pub domains: Option<Vec<String>>,
}

impl App {
    pub fn run(&self, networks: &Vec<&Network>) {
        let mut args = vec!["run", "-d", "--name", self.container_name.as_str()];

        for env_var in &self.env_vars {
            args.push("-e");
            args.push(env_var);
        }

        for volume in &self.volumes {
            args.push("-v");
            args.push(volume);
        }

        args.push(self.image.as_str());

        self.stop();
        let Ok(mut command) = Command::new("docker")
            .args(args)
            .stdout(Stdio::null())
            .spawn()
        else {
            return;
        };

        let Ok(_) = command.wait() else {
            return;
        };

        for network in networks {
            let (connect, name) = match network {
                Network::Internal(name) => (true, name),
                Network::Nginx(name) => (self.domains.is_some(), name),
            };
            if connect {
                run_script!(format!(
                    "docker network connect {} {}",
                    name, self.container_name
                ))
                .unwrap_or_default();
            }
        }
    }

    pub fn stop(&self) {
        run_script!(format!("docker stop {}", self.container_name)).unwrap_or_default();
        run_script!(format!("docker rm {}", self.container_name)).unwrap_or_default();
    }

    pub fn is_running(&self) -> bool {
        let output = run_script!(format!("docker ps -q -f name={}", self.container_name));
        if let Ok((_, output, _)) = output {
            !output.is_empty()
        } else {
            false
        }
    }

    pub fn update(&mut self, new: App) {
        self.name = new.name;
        self.image = new.image;
        self.container_name = new.container_name;
        self.ports = new.ports;
        self.env_vars = new.env_vars;
        self.volumes = new.volumes;
        self.depends_on = new.depends_on;
        self.domains = new.domains;
    }

    pub fn from_cli(args: &ArgMatches, project: &String) -> Vec<Self> {
        let mut apps = Self::collect_flags::<String>(args, "app");
        let mut image_list = Self::collect_flags::<String>(args, "image");
        let mut env_list = Self::collect_flags::<String>(args, "env");
        let mut port_list = Self::collect_flags::<String>(args, "port");
        let mut volume_list = Self::collect_flags::<String>(args, "volume");
        let mut depends_on_list = Self::collect_flags::<String>(args, "depends-on");
        let mut domain_list = Self::collect_flags::<String>(args, "domain");

        let mut app_list: Vec<App> = vec![];
        while let Some(app) = apps.pop() {
            let mut image: String = String::new();
            let mut env_vars: Vec<String> = vec![];
            let mut ports: Vec<String> = vec![];
            let mut volumes: Vec<String> = vec![];
            let mut depends_on: Vec<String> = vec![];
            let mut domains: Vec<String> = vec![];

            while let Some(image_name) = image_list.pop() {
                if image_name.index > app.index {
                    if !image.is_empty() {
                        panic!("Error: App cannot have more than one image");
                    }
                    image = image_name.value;
                } else {
                    image_list.push(image_name);
                    break;
                }
            }

            if image.is_empty() {
                panic!("Error: App must have an image");
            }

            while let Some(env) = env_list.pop() {
                if env.index > app.index {
                    env_vars.push(env.value);
                } else {
                    env_list.push(env);
                    break;
                }
            }

            while let Some(port) = port_list.pop() {
                if port.index > app.index {
                    ports.push(port.value);
                } else {
                    port_list.push(port);
                    break;
                }
            }

            while let Some(volume) = volume_list.pop() {
                if volume.index > app.index {
                    volumes.push(volume.value);
                } else {
                    volume_list.push(volume);
                    break;
                }
            }

            while let Some(depends_on_app) = depends_on_list.pop() {
                if depends_on_app.index > app.index {
                    depends_on.push(depends_on_app.value);
                } else {
                    depends_on_list.push(depends_on_app);
                    break;
                }
            }

            while let Some(domain) = domain_list.pop() {
                if domain.index > app.index {
                    domains.push(domain.value);
                } else {
                    domain_list.push(domain);
                    break;
                }
            }

            let domains = if domains.is_empty() {
                None
            } else {
                Some(domains)
            };

            app_list.push(App {
                name: app.value.to_owned(),
                image,
                container_name: format!("nbot_{}_{}", project, app.value),
                env_vars,
                ports,
                volumes,
                depends_on,
                domains,
            });
        }

        // Validate
        if !image_list.is_empty() {
            panic!("Error: Invalid image outside of app definition");
        } else if !env_list.is_empty() {
            panic!("Error: Invalid environment variable outside of app definition");
        } else if !port_list.is_empty() {
            panic!("Error: Invalid port outside of app definition");
        } else if !volume_list.is_empty() {
            panic!("Error: Invalid volume outside of app definition");
        } else if !depends_on_list.is_empty() {
            panic!("Error: Invalid depends-on outside of app definition");
        } else if !domain_list.is_empty() {
            panic!("Error: Invalid domain outside of app definition");
        }

        for app in &app_list {
            // ensure app name is unique
            let mut count = 0;
            for other_app in &app_list {
                if app.name == other_app.name {
                    count += 1;
                }
            }
            if count > 1 {
                panic!("Error: App name must be unique");
            }

            // ensure every app depends on an app that exists, but not itself
            for dependency in &app.depends_on {
                if dependency == &app.name {
                    panic!("Error: App cannot depend on itself");
                }
                let mut found = false;
                for other_app in &app_list {
                    if dependency == &other_app.name {
                        found = true;
                        break;
                    }
                }
                if !found {
                    panic!("Error: App depends on an app that does not exist");
                }
            }

            // ensure two apps don't depend on each other
            for other_app in &app_list {
                if app.name == other_app.name {
                    continue;
                }
                for dependency in &other_app.depends_on {
                    if dependency == &app.name && app.depends_on.contains(&other_app.name) {
                        panic!("Error: Two apps cannot depend on each other");
                    }
                }
            }
        }

        app_list
    }

    fn collect_flags<T>(args: &ArgMatches, flag: &'static str) -> Vec<Flag<T>>
    where
        T: Clone + std::marker::Send + std::marker::Sync + 'static,
    {
        let indices: Vec<usize> = args.indices_of(flag).unwrap_or_default().collect();
        let values: Vec<T> = args.get_many(flag).unwrap_or_default().cloned().collect();

        let mut flags: Vec<Flag<T>> = vec![];
        for i in 0..indices.len() {
            flags.push(Flag {
                index: indices[i],
                value: values[i].clone(),
            });
        }
        flags
    }
}

#[derive(Debug)]
struct Flag<T> {
    #[allow(dead_code)]
    index: usize,
    #[allow(dead_code)]
    value: T,
}
