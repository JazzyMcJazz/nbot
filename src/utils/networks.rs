use crate::docker;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Network {
    Internal(String),
    Nginx(String),
}

impl Network {
    pub fn internal_from_project(project_name: &String) -> Self {
        Network::Internal(format!("nbot_{}_net", project_name))
    }

    pub fn nginx_from_project(project_name: &String) -> Self {
        Network::Nginx(format!("nbot_nginx_{}_net", project_name))
    }

    pub async fn create(self) -> Self {
        let network_name = match &self {
            Network::Internal(name) => name,
            Network::Nginx(name) => name,
        };

        let exists = docker::network::exists(network_name).await;
        if !exists {
            let _ = docker::network::create(network_name).await;
        }

        self
    }

    pub async fn remove(self) -> Self {
        let network_name = match &self {
            Network::Internal(name) => name,
            Network::Nginx(name) => name,
        };

        docker::network::remove(network_name).await;

        self
    }
}
