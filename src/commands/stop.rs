use crate::models::Project;

pub struct Stop;

impl Stop {
    pub async fn project(project: Project) {
        for app in project.apps {
            app.stop().await;
        }
    }
}
