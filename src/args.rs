use crate::utils::version::Version;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

pub fn get_matches() -> ArgMatches {
    Command::new("nbot")
        .about("An orchestration tool for managing docker containers behind an Nginx reverse proxy.")
        .version(Version::get().as_str())
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
                        .about("Stops the nginx container")
                        .arg(
                            Arg::new("remove")
                                .short('r')
                                .long("rm")
                                .action(ArgAction::SetTrue)
                                .help("Removes nginx container after stopping it")
                        )
                )
        )
        .subcommand(
            Command::new("run")
                .about("Creates or updates a project.\nRun \"nbot run --help\" for more information.")
                .arg(
                    Arg::new("name")
                        .value_parser(value_parser!(String))
                        .short('n')
                        .long("name")
                        .help("Name of the project (required)")
                        .required(true)
                )
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .help("Force the recreation of an existing project. Hint: use in CI/CD pipeline. (optional, defaults to false)")
                        .required(false)
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("app")
                        .short('a')
                        .long("app")
                        .help("Name of an app to add to the project (at least 1 required)")
                        .required(true)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("image")
                        .short('i')
                        .long("image")
                        .help("Image to use for the app (required, exactly 1 per app)")
                        .required(true)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("env")
                        .short('e')
                        .long("env")
                        .help("Environment variables to add to the app (optional, multiple allowed per app)")
                        .required(false)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port where app is running. Nginx uses this port (optional, max 1 per app)")
                        .required(false)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("volume")
                        .short('v')
                        .long("volume")
                        .help("Volumes to add to the app (optional, multiple allowed per app)")
                        .required(false)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("depends-on")
                        .short('d')
                        .long("depends-on")
                        .help("Apps that this app depends on (optional, multiple allowed per app)")
                        .required(false)
                        .action(ArgAction::Append)
                )
                .arg(
                    Arg::new("domain")
                        .short('o')
                        .long("domain")
                        .help("Domain to use for the project. Exposes the app to the internet (optional, multiple allowed per app)")
                        .required(false)
                )
                .arg(
                    Arg::new("email")
                        .short('m')
                        .long("email")
                        .help("Email to use for the project (required if --domain is used)")
                        .required(false)
                )
                .arg(
                    Arg::new("openssl")
                        .short('s')
                        .long("openssl")
                        .help("Use OpenSSL instead of Let's Encrypt for SSL certificates (optional, defaults to false)")
                        .required(false)
                        .action(ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("privileged")
                        .short('r')
                        .long("privileged")
                        .help("Run the container in privileged mode (optional, defaults to false)")
                        .required(false)
                        .action(ArgAction::Append)
                        .value_parser(value_parser!(bool))
                )
                .arg(
                    Arg::new("network-alias")
                        .short('l')
                        .long("network-alias")
                        .help("Network aliases to add to the app (optional, multiple allowed per app)")
                        .required(false)
                        .action(ArgAction::Append)
                )
        )
        .subcommand(
            Command::new("start")
                .about("Starts containers an existing project")
                .arg(
                    Arg::new("project")
                        .value_parser(value_parser!(String))
                        .help("Name of the project to start (required)")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("stop")
                .about("Stops containers in an existing project")
                .arg(
                    Arg::new("project")
                        .value_parser(value_parser!(String))
                        .help("Name of the project to stop (required)")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("rm")
                .about("Removes a project")
                .arg(
                    Arg::new("project")
                        .value_parser(value_parser!(String))
                        .action(ArgAction::Append)
                        .help("Name(s) of the project to remove (required)")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("status")
                .about("Displays the status of all projects")
        )
        .subcommand(
            Command::new("reset")
                .about("Removes all nginx volumes (including certificates), project containers, networks and configurations. Use with caution!")
                .arg(
                    Arg::new("force")
                        .short('f')
                        .long("force")
                        .help("Skip confirmation prompt (optional, defaults to false)")
                        .required(false)
                        .action(ArgAction::SetTrue)
                )
        )
        .get_matches()
}
