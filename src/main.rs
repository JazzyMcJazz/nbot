use configs::app_state::AppState;
use once_cell::sync::Lazy;

mod args;
mod commands;
mod configs;
mod files;
mod models;
mod utils;

static APP_STATE: Lazy<AppState> = Lazy::new(AppState::from_storage);

fn main() {
    let _app_state = AppState::from_storage();
    let matches = args::get_matches();
    commands::process_matches(matches);
}
