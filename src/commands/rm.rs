use clap::ArgMatches;

use crate::APP_STATE;

pub struct Rm;

impl Rm {
    pub fn projects(args: &ArgMatches) {
        let mut state = APP_STATE.clone();

        let projects: Vec<String> = args
            .get_many("project")
            .unwrap_or_default()
            .cloned()
            .collect();

        let mut projects_to_remove = vec![];
        let mut projects_to_keep = vec![];

        for project in state.projects {
            if projects.contains(&project.name) {
                projects_to_remove.push(project);
            } else {
                projects_to_keep.push(project);
            }
        }

        for project in projects_to_remove {
            for app in &project.apps {
                if app.is_running() {
                    app.stop();
                }
            }
        }

        state.projects = projects_to_keep;
        state.save();
    }
}
