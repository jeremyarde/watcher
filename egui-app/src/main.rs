use background::app::AppState;
use eframe::egui;
use log::info;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    unsafe { std::env::set_var("RUST_LOG", rust_log) };

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    start_puffin_server(); // NOTE: you may only want to call this if the users specifies some flag or clicks a button!

    info!("Starting app");

    let appstate = Arc::new(RwLock::new(AppState::new()));
    let appstate_clone = Arc::clone(&appstate);
    thread::spawn(move || {
        puffin::profile_scope!("run_check");

        let mut last_check = Instant::now();
        loop {
            if last_check.elapsed() >= Duration::from_secs(1) {
                appstate_clone.write().run_check();
                last_check = Instant::now();
            }
            // Sleep for a short time to avoid busy-waiting, but not long enough to cause lag
            // thread::sleep(Duration::from_millis(50));
        }
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 50.0])
            // .with_titlebar_buttons_shown(true)
            // .with_title_shown(false)
            .with_taskbar(false)
            .with_always_on_top()
            .with_decorations(true)
            .with_transparent(true)
            .with_position([1000.0, 100.0]),

        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, appstate)))),
    )
}

// #[derive(Deserialize, Serialize)]
#[derive(Debug)]
pub struct MyApp {
    appstate: Arc<RwLock<AppState>>,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, appstate: Arc<RwLock<AppState>>) -> Self {
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

        MyApp { appstate: appstate }
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
            puffin::profile_scope!("central_panel");
            let appstate = self.appstate.read();
            // Display a single line of text with the current session details and timer
            let session_text = if let Some(session) = appstate.active_session.clone() {
                puffin::profile_scope!("session_text");

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

            ui.label(format!("{:#?}", appstate.stats.clone()))
        });
    }
}

fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("127.0.0.1:8585") {
        Ok(puffin_server) => {
            log::info!("Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            std::process::Command::new("puffin_viewer")
                .arg("--url")
                .arg("127.0.0.1:8585")
                .spawn()
                .ok();

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[expect(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            log::error!("Failed to start puffin server: {err}");
        }
    };
}
