use serde::Deserialize;

use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::scripts::{ScriptType, load_scripts};

#[derive(Clone)]
pub struct Config {
    pub break_duration: Duration,
    pub break_interval: Duration,
    pub eye_strain_break_interval: Duration,
    pub eye_strain_break_duration: Duration,
}

// #[derive(Deserialize, Serialize)]
pub struct AppState {
    pub last_break_at: Instant,
    pub last_eye_strain_break_at: Instant,
    pub sessions: Arc<Mutex<Vec<Session>>>,
    pub config: Config,
    pub label: String,
    pub value: f32,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let config = Config {
            break_duration: Duration::from_secs(10),
            break_interval: Duration::from_secs(60),
            eye_strain_break_interval: Duration::from_secs(60),
            eye_strain_break_duration: Duration::from_secs(10),
        };
        let sessions = Arc::new(Mutex::new(vec![]));
        let sessions_clone = sessions.clone();
        let config_clone = config.clone();
        std::thread::spawn(move || {
            let scripts = load_scripts();
            let mut active_session = create_session(&scripts);
            loop {
                let now = Instant::now();
                let session = create_session(&scripts);
                if session.app != active_session.app {
                    active_session.end_at = now;
                    if let Ok(mut sessions) = sessions_clone.lock() {
                        sessions.push(active_session);
                    }
                    active_session = session;
                    active_session.start_at = now;
                }
                std::thread::sleep(Duration::from_millis(2000));
            }
        });
        Self {
            last_break_at: Instant::now(),
            last_eye_strain_break_at: Instant::now(),
            sessions,
            config,
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub start_at: Instant,
    pub end_at: Instant,
    pub app: String,
    pub window_title: String,
}

pub struct Stats {
    // foreground_time
}

fn create_session(scripts: &HashMap<ScriptType, String>) -> Session {
    let app = Command::new("osascript")
        .arg("-e")
        .arg(scripts.get(&ScriptType::ForegroundApp).unwrap())
        .output()
        .expect("Failed to execute script");

    let window_title = Command::new("osascript")
        .arg("-e")
        .arg(scripts.get(&ScriptType::ForegroundWindowTitle).unwrap())
        .output()
        .expect("Failed to execute script");

    let app = String::from_utf8_lossy(&app.stdout).to_string();
    let window_title = String::from_utf8_lossy(&window_title.stdout).to_string();

    Session {
        start_at: Instant::now(),
        end_at: Instant::now(),
        app,
        window_title,
    }
}
