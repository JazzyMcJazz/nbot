use bollard::Docker;
use configs::app_state::AppState;
use once_cell::sync::Lazy;

mod args;
mod commands;
mod configs;
mod docker;
mod models;
mod nginx_files;
mod utils;

static APP_STATE: Lazy<AppState> = Lazy::new(AppState::from_storage);
static DOCKER: Lazy<Docker> = Lazy::new(|| Docker::connect_with_local_defaults().unwrap());

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match DOCKER.ping().await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error connecting to Docker: {}", e);
            std::process::exit(1);
        }
    }

    let _app_state = AppState::from_storage();
    let matches = args::get_matches();
    commands::process_matches(matches).await;
}
