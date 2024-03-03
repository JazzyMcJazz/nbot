use crate::APP_STATE;

use super::{nginx::Nginx, run::Run};

pub struct UpDown;

impl UpDown {
    pub async fn up() {
        let state = APP_STATE.to_owned();

        if !Nginx::is_running().await {
            Nginx::run(false).await;
        }

        for project in state.projects {
            Run::project(project, true).await; // TODO: error handling
        }
    }

    pub async fn down() {
        let state = APP_STATE.to_owned();
        Nginx::stop(true).await;

        for project in state.projects {
            for app in project.apps {
                app.stop().await;
                app.remove().await;
            }
        }
    }
}
