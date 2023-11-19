use clap::{Arg, ArgAction, Command};

mod commands;
mod files;
mod utils;

fn main() {
    let matches = Command::new("nbot")
        .about("An orchestration tool for managing docker containers behind an Nginx reverse proxy.")
        .version("0.0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("JazzyMcJazz")
        .subcommand(
            Command::new("up")
                .about("Starts all containers")
                .subcommand_required(false)
                .arg_required_else_help(false)
        )
        .subcommand(
            Command::new("down")
                .about("Stops all containers")
                .subcommand_required(false)
                .arg(
                    Arg::new("remove")
                        .short('r')
                        .long("rm")
                        .action(ArgAction::SetTrue)
                        .help("Removes all containers after stopping them")
                )
        )
        .subcommand(
            Command::new("docker")
                .about("Manage docker installation.")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("install")
                        .about("Installs docker (Ubuntu only)")
                )
                .subcommand(
                    Command::new("remove")
                        .about("Removes docker (Ubuntu)")
                )
        )
        .subcommand(
            Command::new("nginx")
                .about("Manage nginx installation.")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("run")
                        .about("Runs nginx in Docker")
                        .arg(
                            Arg::new("build")
                                .short('b')
                                .long("build")
                                .action(ArgAction::SetTrue)
                                .help("Builds nbot/nginx image before running it (will remove existing image)")
                        )
                )
                .subcommand(
                    Command::new("stop")
                        .about("Stops and removes nginx container")
                        .arg(
                            Arg::new("remove")
                                .short('r')
                                .long("rm")
                                .action(ArgAction::SetTrue)
                                .help("Removes nginx container after stopping it")
                        )
                )
        )
        .get_matches();

    commands::process_args(matches);
}
