use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use super::app::App;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub apps: Vec<App>,
}

impl Project {
    pub fn from_cli(args: &ArgMatches) -> Self {
        let name = args
            .get_one::<String>("name")
            .expect("No project name provided")
            .to_owned();
        let apps = App::from_cli(args, &name);
        Self { name, apps }
    }
}
