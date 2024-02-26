use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use crate::APP_STATE;

use super::app::App;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub apps: Vec<App>,
}

impl Project {
    pub fn from_cli_run(args: &ArgMatches) -> Self {
        let name = args
            .get_one::<String>("name")
            .expect("No project name provided")
            .to_owned();
        let apps = App::from_cli(args, &name);
        Self { name, apps }
    }

    pub fn from_cli_start(args: &ArgMatches) -> Self {
        let name = args
            .get_one::<String>("project")
            .expect("No project name provided")
            .to_owned();
        let project = APP_STATE
            .projects
            .iter()
            .find(|project| project.name == name)
            .expect("Project not found");

        project.clone()
    }
}
