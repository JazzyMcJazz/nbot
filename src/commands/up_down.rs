use crate::{utils::spinner::Spinner, APP_STATE};

use super::nginx::Nginx;

pub struct UpDown;

impl UpDown {
    pub fn up() {
        let mut spinner = Spinner::new();

        let state = APP_STATE.to_owned();

        spinner.start("Nginx: ".to_owned());
        Nginx::run(false, true);
        spinner.stop("Nginx: OK".to_owned());

        for project in state.projects {
            for app in project.apps {
                spinner.start(format!("{}: ", app.name));
                if !app.is_running() {
                    app.run(&vec![]);
                }
                spinner.stop(format!("{}: OK", app.name));
            }
        }
    }

    pub fn down() {
        let mut spinner = Spinner::new();

        let state = APP_STATE.to_owned();

        spinner.start("Nginx: ".to_owned());
        Nginx::stop(true, true);
        spinner.stop("Nginx: stopped".to_owned());

        for project in state.projects {
            for app in project.apps {
                spinner.start(format!("{}: ", app.name));
                if app.is_running() {
                    app.stop();
                }
                spinner.stop(format!("{}: stopped", app.name));
            }
        }
    }
}
