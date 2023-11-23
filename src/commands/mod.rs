use clap::ArgMatches;

mod nginx;
mod rm;
mod run;
mod up_down;

use nginx::Nginx;
use rm::Rm;
use run::Run;
use up_down::UpDown;

use crate::models::Project;

pub fn process_matches(args: ArgMatches) {
    match args.subcommand() {
        Some(("nginx", sync_matches)) => {
            Nginx::process_matches(sync_matches);
        }
        Some(("up", _)) => {
            UpDown::up();
        }
        Some(("down", _)) => {
            UpDown::down();
        }
        Some(("run", args)) => {
            let project = Project::from_cli(args);
            Run::project(project);
        }
        Some(("rm", args)) => {
            Rm::projects(args);
        }
        _ => unreachable!(),
    }
}
