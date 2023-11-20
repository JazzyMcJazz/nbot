use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use super::app::App;

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    name: String,
    apps: Vec<App>,
}

impl Project {
    pub fn from_cli(args: &ArgMatches) -> Self {
        let name = args
            .get_one::<String>("project")
            .expect("No project name provided")
            .to_owned();
        let apps = App::from_cli(args, &name);
        Self { name, apps }
    }
}
