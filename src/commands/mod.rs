use clap::ArgMatches;

mod nginx;

use nginx::Nginx;

use crate::{configs::project::Project, utils::spinner::Spinner};

pub fn process_matches(args: ArgMatches) {
    match args.subcommand() {
        Some(("nginx", sync_matches)) => {
            Nginx::process_matches(sync_matches);
        }
        Some(("up", _)) => {
            let mut spinner = Spinner::new();
            spinner.start("");
            Nginx::run(false, true);
            spinner.stop("All containers started.");
        }
        Some(("down", args)) => {
            let remove = args.get_flag("remove");
            let mut spinner = Spinner::new();
            spinner.start("");
            Nginx::stop(remove, true);
            spinner.stop("All containers stopped.");
        }
        Some(("run", args)) => {
            let apps = Project::from_cli(args);
            dbg!(apps);
        }
        _ => unreachable!(),
    }
}
