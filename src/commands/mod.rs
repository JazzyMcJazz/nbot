use clap::ArgMatches;

mod nginx;
mod rm;
mod run;
mod start;
mod status;
mod stop;
mod up_down;

use nginx::Nginx;
use rm::Rm;
use run::Run;
use start::Start;
use status::Status;
use stop::Stop;
use up_down::UpDown;

use crate::models::Project;

pub async fn process_matches(args: ArgMatches) {
    match args.subcommand() {
        Some(("nginx", sync_matches)) => {
            Nginx::process_matches(sync_matches).await;
        }
        Some(("up", _)) => {
            UpDown::up().await;
        }
        Some(("down", _)) => {
            UpDown::down().await;
        }
        Some(("run", args)) => {
            let project = Project::from_cli_run(args);
            let force = args.get_flag("force");
            Run::project(project, force).await;
        }
        Some(("start", args)) => {
            let project = Project::from_cli_start(args);
            Start::project(project).await;
        }
        Some(("stop", args)) => {
            let project = Project::from_cli_start(args);
            Stop::project(project).await;
        }
        Some(("rm", args)) => {
            Rm::projects(args).await;
        }
        Some(("status", _)) => {
            Status::new().await.display();
        }
        _ => unreachable!(),
    }
}
