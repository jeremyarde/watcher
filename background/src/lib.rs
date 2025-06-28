pub mod app;
pub mod pomodoro;
pub mod scripts;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use app::AppState;

use crate::{
    app::{Config, create_session},
    scripts::load_scripts,
};

pub fn create_process() -> AppState {
    AppState::new()
}

pub fn create_appstate() -> AppState {
    AppState::new()
}

#[cfg(test)]
mod tests {
    use super::*;
}
