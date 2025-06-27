// use serde::Deserialize;

use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::scripts::{ScriptType, load_scripts};

#[derive(Clone, Debug)]
pub struct Config {
    pub break_duration: Duration,
    pub break_interval: Duration,
    pub eye_strain_break_interval: Duration,
    pub eye_strain_break_duration: Duration,
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub total_time_per_app: HashMap<String, Duration>,
    pub session_count_per_app: HashMap<String, usize>,
    // Add more fields as needed
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub last_break_at: Instant,
    pub last_eye_strain_break_at: Instant,
    pub sessions: Vec<Session>,
    pub active_session: Option<Session>,
    pub config: Config,
    pub label: String,
    pub value: f32,
    pub scripts: HashMap<ScriptType, String>,
    pub stats: Stats,
}

impl AppState {
    pub fn run_check(&self) -> AppState {
        let mut new_state = self.clone();
        let now = Instant::now();
        let session = create_session(&new_state.scripts);

        if let Some(active) = new_state.active_session.as_ref() {
            if session.app != active.app {
                if let Some(active_mut) = new_state.active_session.as_mut() {
                    active_mut.end_at = now;
                }
                new_state
                    .sessions
                    .push(new_state.active_session.take().unwrap());
                new_state.active_session = Some(session);
                if let Some(active_mut) = new_state.active_session.as_mut() {
                    active_mut.start_at = now;
                }
            } else {
                if let Some(active_mut) = new_state.active_session.as_mut() {
                    active_mut.end_at = now;
                }
            }
        } else {
            new_state.active_session = Some(session);
            if let Some(active_mut) = new_state.active_session.as_mut() {
                active_mut.start_at = now;
            }
        }
        new_state.stats = new_state.compute_stats();
        new_state
    }

    pub fn compute_stats(&self) -> Stats {
        let mut total_time_per_app = HashMap::new();
        let mut session_count_per_app = HashMap::new();

        for session in &self.sessions {
            let duration = session.end_at.duration_since(session.start_at);
            *total_time_per_app
                .entry(session.app.clone())
                .or_insert(Duration::ZERO) += duration;
            *session_count_per_app
                .entry(session.app.clone())
                .or_insert(0) += 1;
        }

        Stats {
            total_time_per_app,
            session_count_per_app,
        }
    }

    pub fn new() -> Self {
        let config = Config {
            break_duration: Duration::from_secs(10),
            break_interval: Duration::from_secs(60),
            eye_strain_break_interval: Duration::from_secs(60),
            eye_strain_break_duration: Duration::from_secs(10),
        };
        let sessions = vec![];
        // let sessions_clone = sessions.clone();
        // let config_clone = config.clone();
        // std::thread::spawn(move || {
        //     let scripts = load_scripts();
        //     let mut active_session = create_session(&scripts);
        //     loop {
        //         let now = Instant::now();
        //         let session = create_session(&scripts);
        //         if session.app != active_session.app {
        //             active_session.end_at = now;
        //             if let Ok(mut sessions) = sessions_clone.lock() {
        //                 sessions.push(active_session);
        //             }
        //             active_session = session;
        //             active_session.start_at = now;
        //         }
        //         std::thread::sleep(Duration::from_millis(2000));
        //     }
        // });

        let active_session = create_session(&load_scripts());

        Self {
            active_session: Some(active_session),
            last_break_at: Instant::now(),
            last_eye_strain_break_at: Instant::now(),
            sessions,
            config,
            label: "Hello World!".to_owned(),
            value: 2.7,
            scripts: load_scripts(),
            stats: Stats {
                total_time_per_app: HashMap::new(),
                session_count_per_app: HashMap::new(),
            },
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

pub fn create_session(scripts: &HashMap<ScriptType, String>) -> Session {
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
