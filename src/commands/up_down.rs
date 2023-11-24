use crate::{utils::spinner::Spinner, APP_STATE};

use super::{nginx::Nginx, run::Run};

pub struct UpDown;

impl UpDown {
    pub fn up() {
        let mut spinner = Spinner::new();

        let state = APP_STATE.to_owned();

        if !Nginx::is_running() {
            spinner.start("Nginx: ".to_owned());
            Nginx::run(false, true);
            spinner.stop("Nginx: OK".to_owned());
        }

        for project in state.projects {
            // let project_name = project.name.clone();
            // spinner.start(format!("{}: ", &project_name));
            Run::project(project); // TODO: error handling
                                   // spinner.stop(format!("{}: OK", project_name));
                                   // for app in project.apps {
                                   //     if !app.is_running() {
                                   //         spinner.start(format!("{}: ", app.name));
                                   //         app.run(&vec![]);
                                   //         spinner.stop(format!("{}: OK", app.name));
                                   //     }
                                   // }
        }
    }

    pub fn down() {
        let mut spinner = Spinner::new();

        let state = APP_STATE.to_owned();

        if Nginx::is_running() {
            spinner.start("Nginx: ".to_owned());
            Nginx::stop(true, true);
            spinner.stop("Nginx: stopped".to_owned());
        }

        for project in state.projects {
            for app in project.apps {
                if app.is_running() {
                    spinner.start(format!("{}: ", app.name));
                    app.stop();
                    spinner.stop(format!("{}: stopped", app.name));
                }
            }
        }
    }
}
