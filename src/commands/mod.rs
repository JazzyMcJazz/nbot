use clap::ArgMatches;

mod docker;
mod nginx;

use nginx::Nginx;

use crate::utils::spinner::Spinner;

pub fn process_args(args: ArgMatches) {
    match args.subcommand() {
        Some(("docker", sync_matches)) => {
            docker::process(sync_matches);
        }
        Some(("nginx", sync_matches)) => {
            Nginx::process_args(sync_matches);
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
        _ => unreachable!(),
    }
}
