use crate::models::Project;

pub struct Start;

impl Start {
    pub async fn project(project: Project) {
        for app in project.apps {
            app.start().await;
        }
    }
}
