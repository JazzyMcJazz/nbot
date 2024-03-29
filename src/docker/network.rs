use std::collections::HashMap;

use bollard::{
    network::{
        ConnectNetworkOptions, CreateNetworkOptions, DisconnectNetworkOptions, ListNetworksOptions,
    },
    secret::ContainerSummary,
};

use crate::{utils::networks::Network, APP_STATE, DOCKER};

pub async fn is_connected(container_id: &str, network: &Network) -> bool {
    let network_name = match network {
        Network::Internal(name) => name,
        Network::Nginx(name) => name,
    };

    let connected = DOCKER.inspect_container(container_id, None).await;
    match connected {
        Ok(container) => {
            let networks = container.network_settings.unwrap().networks.unwrap();
            networks.contains_key(network_name)
        }
        Err(e) => {
            eprintln!("Error checking if container is connected to network: {}", e);
            false
        }
    }
}

pub async fn exists(name: &str) -> bool {
    let filters = HashMap::from([("name", vec![name])]);
    let options = Some(ListNetworksOptions { filters });
    let networks = DOCKER.list_networks(options).await;
    match networks {
        Ok(networks) => {
            for network in networks {
                if network.name == Some(name.to_owned()) {
                    return true;
                }
            }
            false
        }
        Err(e) => {
            eprintln!("Error listing networks: {}", e);
            false
        }
    }
}

pub async fn find_ids_by_container(container: &str) -> Vec<String> {
    let details = DOCKER.inspect_container(container, None).await;
    match details {
        Ok(details) => {
            let mut networks = vec![];
            if let Some(settings) = details.network_settings {
                if let Some(nets) = settings.networks {
                    for (name, value) in nets {
                        if name.starts_with(&APP_STATE.network_prefix) {
                            if let Some(network_id) = value.network_id {
                                networks.push(network_id);
                            }
                        }
                    }
                }
            }
            networks
        }
        Err(_) => vec![],
    }
}

pub async fn create(name: &str) -> bool {
    let options = CreateNetworkOptions {
        name,
        ..Default::default()
    };

    let network = DOCKER.create_network(options).await;
    match network {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error creating network: {}", e);
            false
        }
    }
}

pub async fn remove(name: &str) -> bool {
    let removed = DOCKER.remove_network(name).await;
    match removed {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error removing network: {}", e);
            false
        }
    }
}

pub async fn remove_many(names: Vec<String>) {
    for name in names {
        if DOCKER.remove_network(&name).await.is_ok() {
            println!("{}", name);
        }
    }
}

pub async fn connect(container_id: &str, network: &Network) -> bool {
    let network_name = match network {
        Network::Internal(name) => name,
        Network::Nginx(name) => name,
    };

    let config = ConnectNetworkOptions {
        container: container_id,
        ..Default::default()
    };

    let connected = DOCKER.connect_network(network_name, config).await;
    match connected {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error connecting container to network: {}", e);
            false
        }
    }
}

pub async fn disconnect(container: ContainerSummary, network: &Network) -> bool {
    let network_name = match network {
        Network::Internal(name) => name,
        Network::Nginx(name) => name,
    };

    let config = DisconnectNetworkOptions {
        container: container.id.unwrap(),
        ..Default::default()
    };

    let disconnected = DOCKER.disconnect_network(network_name, config).await;
    match disconnected {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error disconnecting container from network: {}", e);
            false
        }
    }
}
