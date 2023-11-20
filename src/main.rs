use configs::app_state::AppState;

mod args;
mod commands;
mod configs;
mod files;
mod utils;

fn main() {
    let _app_state = AppState::new();
    let matches = args::get_matches();
    commands::process_matches(matches);
}
