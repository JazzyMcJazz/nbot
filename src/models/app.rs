use std::collections::{HashMap, HashSet};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::{docker, utils::networks::Network, APP_STATE};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    pub name: String,
    pub image: String,
    pub container_name: String,
    pub port: Option<String>,
    pub env_vars: Vec<String>,
    pub volumes: Vec<String>,
    depends_on: Vec<String>,
    pub domains: Option<Vec<String>>,
    pub email: Option<String>,
    pub openssl: Option<bool>,
    pub privileged: bool,
    pub network_aliases: Vec<String>,
}

impl App {
    pub async fn run(&self, networks: &Vec<&Network>) -> bool {
        if self.is_using_latest_image().await && self.is_running().await {
            return true;
        }

        self.stop().await;
        self.remove().await;

        let result = docker::containers::create_from_app(self).await;
        let container = match result {
            Ok(container) => container,
            Err(e) => {
                eprintln!("Error creating container: {}", e);
                return false;
            }
        };

        for network in networks {
            let (connect, _) = match network {
                Network::Internal(name) => (true, name),
                Network::Nginx(name) => (self.domains.is_some(), name),
            };

            if connect {
                let connected = docker::network::is_connected(container.id.as_str(), network).await;
                if !connected {
                    let did_connect =
                        docker::network::connect(container.id.as_str(), network).await;
                    if !did_connect {
                        eprintln!("Error connecting container to network");
                        return false;
                    }
                }
            }
        }

        let started = docker::containers::start(container.id.as_str()).await;
        if !started {
            eprintln!("Error starting container");
            return false;
        }

        true
    }

    pub async fn start(&self) -> bool {
        let container = docker::containers::find_by_name(self.container_name.as_str()).await;
        if let Some(container) = container {
            dbg!("found container");
            if let Some(state) = container.state {
                if state == "running" {
                    return true;
                }
            }
            let started = docker::containers::start(self.container_name.as_str()).await;
            if !started {
                eprintln!("Error starting container");
                return false;
            }
        } else {
            let container = docker::containers::create_from_app(self).await;
            let Ok(_) = container else {
                eprintln!("Error creating container");
                return false;
            };

            let started = docker::containers::start(self.container_name.as_str()).await;
            if !started {
                eprintln!("Error starting container");
                return false;
            }
        };

        true
    }

    pub async fn stop(&self) {
        docker::containers::stop(self.container_name.as_str()).await;
    }

    pub async fn remove(&self) {
        docker::containers::remove(self.container_name.as_str()).await;
    }

    pub async fn is_running(&self) -> bool {
        let container = docker::containers::find_by_name(self.container_name.as_str()).await;
        if let Some(container) = container {
            if let Some(state) = container.state {
                return state == "running";
            }
        }
        false
    }

    async fn is_using_latest_image(&self) -> bool {
        let container = docker::containers::find_by_name(self.container_name.as_str()).await;
        let Some(container) = container else {
            return false;
        };

        let Some(image_id) = container.image_id else {
            return false;
        };

        let image = docker::images::find_by_name(&self.image, None).await;

        if let Some(image) = image {
            return image.id == image_id;
        }
        false
    }

    pub fn from_cli(args: &ArgMatches, project: &String) -> Vec<Self> {
        let mut apps = Self::collect_flags::<String>(args, "app");
        let mut image_list = Self::collect_flags::<String>(args, "image");
        let mut env_list = Self::collect_flags::<String>(args, "env");
        let mut port_list = Self::collect_flags::<String>(args, "port");
        let mut volume_list = Self::collect_flags::<String>(args, "volume");
        let mut depends_on_list = Self::collect_flags::<String>(args, "depends-on");
        let mut domain_list = Self::collect_flags::<String>(args, "domain");
        let mut email_list = Self::collect_flags::<String>(args, "email");
        let mut openssl_list = Self::collect_flags::<bool>(args, "openssl");
        let mut privileged_list = Self::collect_flags::<bool>(args, "privileged");
        let mut network_aliases_list = Self::collect_flags::<String>(args, "network-alias");

        let mut app_list: Vec<App> = vec![];
        while let Some(app) = apps.pop() {
            let mut image: String = String::new();
            let mut env_vars: Vec<String> = vec![];
            let mut virtual_port: Option<String> = None;
            let mut volumes: Vec<String> = vec![];
            let mut depends_on: Vec<String> = vec![];
            let mut domains: Vec<String> = vec![];
            let mut network_aliases: Vec<String> = vec![];
            let mut privileged = false;

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
                    if virtual_port.is_some() {
                        panic!("Error: App cannot have more than one port");
                    }
                    virtual_port = Some(port.value);
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

            while let Some(network_alias) = network_aliases_list.pop() {
                if network_alias.index > app.index {
                    network_aliases.push(network_alias.value);
                } else {
                    network_aliases_list.push(network_alias);
                    break;
                }
            }

            while let Some(privileged_flag) = privileged_list.pop() {
                if privileged_flag.index > app.index {
                    if privileged {
                        panic!("Error: App cannot have more than one privileged flag");
                    }
                    privileged = privileged_flag.value;
                } else {
                    privileged_list.push(privileged_flag);
                    break;
                }
            }

            let mut email: Option<String> = None;
            while let Some(email_address) = email_list.pop() {
                if email_address.index > app.index {
                    if email.is_some() {
                        panic!("Error: App cannot have more than one email");
                    }
                    email = Some(email_address.value);
                } else {
                    email_list.push(email_address);
                    break;
                }
            }

            if email.is_none() && domains.is_some() {
                panic!("Error: App must have an email if it has domains. This is required for SSL certificates.");
            }

            let mut openssl: Option<bool> = None;
            while let Some(openssl_flag) = openssl_list.pop() {
                if openssl_flag.index > app.index {
                    if openssl.is_some() {
                        panic!("Error: App cannot have more than one openssl flag");
                    }
                    openssl = Some(openssl_flag.value);
                } else {
                    openssl_list.push(openssl_flag);
                    break;
                }
            }

            app_list.push(App {
                name: app.value.to_owned(),
                image,
                container_name: format!("{}{}_{}", APP_STATE.container_prefix, project, app.value),
                env_vars,
                port: virtual_port,
                volumes,
                depends_on,
                domains,
                email,
                openssl,
                privileged,
                network_aliases,
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
        } else if !email_list.is_empty() {
            panic!("Error: Invalid email outside of app definition");
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

        app_list.reverse();
        app_list
    }

    pub fn topological_sort_by_dependenceis(apps: &Vec<App>) -> Vec<App> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = Vec::new();
        let mut in_process: HashSet<String> = HashSet::new();

        // Create graph
        for app in apps {
            graph.insert(app.name.to_owned(), app.depends_on.to_owned());
        }

        // DFS function to visit nodes
        fn dfs(
            node: &String,
            graph: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            stask: &mut Vec<String>,
            in_process: &mut HashSet<String>,
        ) {
            if visited.contains(node) {
                return;
            }
            if in_process.contains(node) {
                panic!("Error: Circular dependency detected");
            }

            in_process.insert(node.to_owned());
            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    dfs(neighbor, graph, visited, stask, in_process);
                }
            }

            in_process.remove(node);
            visited.insert(node.to_owned());
            stask.push(node.to_owned());
        }

        // Visit nodes
        for node in graph.keys() {
            if !visited.contains(node) {
                dfs(node, &graph, &mut visited, &mut stack, &mut in_process);
            }
        }

        // Reverse stack
        let sorted: Vec<App> = stack
            .iter()
            .map(|name| {
                apps.iter()
                    .find(|app| app.name == *name)
                    .unwrap()
                    .to_owned()
            })
            .collect();

        sorted
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
