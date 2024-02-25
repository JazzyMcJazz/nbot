use bollard::Docker;
use configs::app_state::AppState;
use once_cell::sync::Lazy;

mod args;
mod commands;
mod configs;
mod docker;
mod files;
mod models;
mod utils;

static APP_STATE: Lazy<AppState> = Lazy::new(AppState::from_storage);
static DOCKER: Lazy<Docker> = Lazy::new(|| Docker::connect_with_local_defaults().unwrap());

#[tokio::main]
async fn main() {
    DOCKER.ping().await.expect("Docker is not running");

    let _app_state = AppState::from_storage();
    let matches = args::get_matches();
    commands::process_matches(matches).await;
}
