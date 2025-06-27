use background::app::AppState;
use eframe::egui;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    info!("Starting app");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 50.0])
            .with_titlebar_buttons_shown(false)
            .with_title_shown(false)
            .with_taskbar(false)
            .with_always_on_top()
            .with_decorations(false)
            .with_position([1000.0, 100.0]),

        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

// #[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct MyApp {
    appstate: Arc<Mutex<AppState>>,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        info!("Creating app");
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            info!("NOT IMPLEMENTED: Loading appstate from storage");
            // return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // let appstate = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        let appstate = Arc::new(Mutex::new(AppState::new()));
        let appstate_clone = Arc::clone(&appstate);
        thread::spawn(move || {
            info!("Starting thread");
            loop {
                if let Ok(mut state) = appstate_clone.lock() {
                    // info!("State before run_check: {:?}", state);
                    state.run_check();
                    // info!("State after run_check: {:?}", state);
                    // info!("Stats: {:?}", state.stats);
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        MyApp { appstate }
    }
}

impl eframe::App for MyApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Display a single line of text with the current session details and timer
            let session_text = if let Some(session) = self
                .appstate
                .lock()
                .ok()
                .and_then(|s| s.active_session.clone())
            {
                // Calculate timer using only start_at and current time
                let now = std::time::Instant::now();
                let duration = now.duration_since(session.start_at);
                let secs = duration.as_secs();
                let mins = secs / 60;
                let secs = secs % 60;
                ctx.request_repaint(); // Ensure the UI updates every frame
                format!(
                    "App: {} | Window: {} | Time: {:02}:{:02}",
                    session.app.trim(),
                    session.window_title.trim(),
                    mins,
                    secs
                )
            } else {
                "No session data available".to_string()
            };
            ui.label(session_text);

            ui.label(format!(
                "{:#?}",
                self.appstate
                    .lock()
                    .ok()
                    .and_then(|state| Some(state.stats.clone()))
            ))
        });
    }
}
