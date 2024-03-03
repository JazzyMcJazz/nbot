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
            .get_one::<String>("name");

        let Some(name) = name else {
            eprintln!("No project name provided");
            std::process::exit(1);
        };

        let apps = App::from_cli(args, &name);
        Self { name: name.to_owned(), apps }
    }

    pub fn from_cli_start(args: &ArgMatches) -> Self {
        let name = args
            .get_one::<String>("project");

        let Some(name) = name else {
            eprintln!("No project name provided");
            std::process::exit(1);
        };

        let project = APP_STATE
            .projects
            .iter()
            .find(|project| project.name == name.to_owned());

        let Some(project) = project else {
            eprintln!("Project not found");
            std::process::exit(1);
        };

        project.to_owned()
    }
}
