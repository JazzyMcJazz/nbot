use bollard::{container::{Config, CreateContainerOptions, ListContainersOptions, NetworkingConfig, StartContainerOptions}, secret::{ContainerCreateResponse, ContainerSummary, EndpointSettings, HostConfig, PortBinding}};
use std::{collections::HashMap, default::Default};

use crate::{models::App, utils::dirs::Dirs, DOCKER};

pub async fn find_by_name(name: &str) -> Option<ContainerSummary> {
    let mut filters = HashMap::new();
    filters.insert("name".to_owned(), vec![name.to_owned()]);

    let options = Some(ListContainersOptions::<String> {
        all: true,
        filters,
        ..Default::default()
    });

    let containers = DOCKER.list_containers(options).await.unwrap();
    if containers.is_empty() {
        return None;
    }
    
    Some(containers[0].clone())
}

pub async fn create_from_app(app: &App) -> Result<ContainerCreateResponse, String> {
    let app = app.clone();
    let options = Some(CreateContainerOptions {
        name: &app.container_name,
        platform: None,
    });

    let port_bindings = {
        if let Some(port) = &app.port {
            Some(HashMap::from([
                (format!("{}/tcp", port), Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(port.to_string()),
                }])),
            ]))
        } else {
            None
        }
    };

    let host_config = Some(HostConfig {
        port_bindings,
        init: Some(true),
        privileged: Some(app.privileged),

        binds: Some(app.volumes.clone()),
        ..Default::default()
    });

    // dbg!(&networks);
    let mut endpoints_config = HashMap::new();
    // for network in networks {
    //     let (connect, name) = match network {
    //         Network::Internal(name) => (true, name),
    //         Network::Nginx(name) => (app.domains.is_some(), name),
    //     };
    //     if connect {
    //         endpoints_config.insert(name.clone(), EndpointSettings::default());
    //     }
    // }

    for network in app.network_aliases {
        endpoints_config.insert(network.clone(), EndpointSettings {
            aliases: Some(vec![network]),
            ..Default::default()
        });
    }

    let networking_config = Some(NetworkingConfig {
        endpoints_config,
    });

    let config = Config {
        image: Some(app.image),
        env: Some(app.env_vars),
        host_config,
        networking_config,
        ..Default::default()
    };
    
    let container = DOCKER.create_container(options, config).await;
    match container {
        Ok(container) => Ok(container),
        Err(e) => Err(e.to_string())
    }
}

pub async fn start(container_id: &str) -> bool {
    
    let started = DOCKER.start_container(container_id, None::<StartContainerOptions<String>>).await;

    match started {
        Ok(_) => {
            println!("{container_id}");
            true
        },
        Err(e) => {
            eprintln!("Error starting container: {}", e);
            false
        }
    }
}

pub async fn stop(container_id: &str) {
    match DOCKER.stop_container(container_id, None).await {
        Ok(_) => {
            println!("{container_id}");
        },
        Err(e) => {
            eprintln!("Error stopping container: {}", e);
        }
    }
}

pub async fn remove(container_id: &str) {
    match DOCKER.remove_container(container_id, None).await {
        Ok(_) => {
            println!("{container_id}");
        },
        Err(e) => {
            eprintln!("Error removing container: {}", e);
        }
    }
}

pub async fn start_nginx() -> bool {
    let image = super::images::find_by_name("nbot/nginx", Some("latest")).await;
    let Some(image) = image else {
        return false;
    };

    let mut filters = HashMap::new();
    filters.insert("ancestor".to_owned(), vec![image.id.to_string()]);

    let options = Some(ListContainersOptions::<String> {
        all: true,
        filters,
        ..Default::default()
    });

    
    let containers = DOCKER.list_containers(options).await.unwrap();
    if containers.is_empty() {
        return false;
    }

    if containers.len() > 1 {
        panic!("More than one instance of nginx is running")
    }

    let nginx = &containers[0];
    let mut started = false;
    if let Some(state) = &nginx.state {
        if state == "running" {
            started = true;
            println!("Nginx is already running");
        } else {
            started = start(nginx.id.as_ref().unwrap()).await;
        }
    }

    started
}

pub async fn run_nginx() -> bool {

    let image = super::images::find_by_name("nbot/nginx", Some("latest")).await
        .expect("Nginx image not found");

    let options = Some(CreateContainerOptions {
        name: "nbot_nginx",
        platform: None,
    });

    let port_bindings = Some(HashMap::from([
        ("443/tcp".to_string(), Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("443".to_string()),
        }])),
        ("80/tcp".to_string(), Some(vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some("80".to_string()),
        }])),
    ]));

    let volume_dir = Dirs::nginx_volumes();
    let binds = Some(vec![
        format!("{}/certs:/etc/letsencrypt:rw", volume_dir),
        format!("{}/conf.d:/etc/nginx/conf.d", volume_dir),
        format!("{}/html:/usr/share/nginx/html", volume_dir),
        format!("{}/static:/static/", volume_dir),
        format!("{}/media:/media/", volume_dir),
    ]);
        
    let host_config = Some(HostConfig {
        port_bindings,
        binds,
        ..Default::default()
    });
    

    let config = Config {
        image: Some(image.id.as_str()),
        host_config,
        ..Default::default()  
    };

    Dirs::init_volumes();
    let container = DOCKER.create_container(options, config).await.expect("Error creating container");
    start(container.id.as_str()).await
}