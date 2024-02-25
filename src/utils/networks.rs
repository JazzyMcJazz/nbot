use run_script::run_script;

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

    pub fn create(self) -> Self {
        let network_name = match &self {
            Network::Internal(name) => name,
            Network::Nginx(name) => name,
        };
        
        let Ok((_, output, _)) = run_script!(format!("docker network ls | grep {}", network_name))
        else {
            return self;
        };

        if output.is_empty()
            && run_script!(format!("docker network create {}", network_name)).is_err()
        {
            return self;
        }

        self
    }

    pub fn remove(self) -> Self {
        let network_name = match &self {
            Network::Internal(name) => name,
            Network::Nginx(name) => name,
        };

        run_script!(format!("docker network rm {}", network_name)).unwrap_or_default();
        self
    }
}
