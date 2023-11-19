use clap::ArgMatches;
use run_script::{run_script, ScriptOptions};

use crate::{files::*, utils::spinner::Spinner};

pub fn process(args: &ArgMatches) {
    match args.subcommand() {
        Some(("install", _)) => {
            install();
        }
        Some(("remove", _)) => {
            remove();
        }
        _ => unreachable!(),
    }
}

fn install() {
    let mut spinner = Spinner::new();
    spinner.start("Installing Docker ");

    let options = ScriptOptions::new();
    let (_, username, _) = run_script!("whoami").unwrap();
    run_script!(DOCKER_INSTALL, vec![username.trim().to_string()], &options).unwrap();

    spinner.stop("Docker installed!");
    println!(
        "Run `sudo usermod -aG docker {}` to add your user to the docker group.",
        username.trim()
    );
    println!("Then log out and back in to apply the changes.");
}

fn remove() {
    let mut spinner = Spinner::new();
    spinner.start("Removing Docker ");

    let (code, _, error) = run_script!(DOCKER_REMOVE).unwrap();

    if code != 0 {
        println!("Error removing docker!");
        println!("{}", error);
        return;
    }

    spinner.stop("Docker removed!");
}
