use arc_swap::ArcSwap;
use background::app::AppState;
use eframe::egui;
use eframe::egui::viewport::ViewportId;
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

    let appstate = Arc::new(ArcSwap::from_pointee(AppState::new()));
    let appstate_clone = Arc::clone(&appstate);
    thread::spawn(move || {
        puffin::profile_scope!("run_check");

        let mut last_check = Instant::now();
        loop {
            if last_check.elapsed() >= Duration::from_secs(1) {
                let new_state = appstate_clone.load().run_check();
                appstate_clone.store(Arc::new(new_state));
                last_check = Instant::now();
            }
            thread::sleep(Duration::from_millis(50));
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
    appstate: Arc<ArcSwap<AppState>>,
    notification_open: bool,
    notification_viewport_id: ViewportId,
    show_immediate_viewport: bool,
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, appstate: Arc<ArcSwap<AppState>>) -> Self {
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

        MyApp {
            appstate,
            notification_open: false,
            notification_viewport_id: ViewportId::from_hash_of("notification_window"),
            show_immediate_viewport: false,
        }
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
                        // Move theme preference buttons here
                        egui::widgets::global_theme_preference_buttons(ui);
                    });
                    ui.add_space(16.0);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            puffin::profile_scope!("central_panel");
            let appstate = self.appstate.load();
            let active_session = appstate.active_session.clone();
            let stats = appstate.stats.clone();

            // Notification button
            // if ui.button("🔔 Notifications").clicked() {
            //     if !self.notification_open() {
            //         self.set_notification_open(true);
            //     }
            // }
            ui.checkbox(
                &mut self.show_immediate_viewport,
                "Show immediate child viewport",
            );

            let mut show_deferred_viewport = self.notification_open;
            ui.checkbox(&mut show_deferred_viewport, "Show deferred child viewport");
            self.notification_open = show_deferred_viewport;

            ui.horizontal(|ui| {
                if let Some(session) = active_session {
                    let now = std::time::Instant::now();
                    let duration = now.duration_since(session.start_at);
                    let secs = duration.as_secs();
                    let mins = secs / 60;
                    let secs = secs % 60;
                    ui.label(egui::RichText::new(format!("{}", session.app.trim())).strong());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(session.window_title.trim()).italics());
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("⏱ {:02}:{:02}", mins, secs)).monospace());
                } else {
                    ui.label("No session");
                }

                // Top 3 apps summary, compact
                let mut app_times: Vec<_> = stats.total_time_per_app.iter().collect();
                app_times.sort_by_key(|&(_, &d)| std::cmp::Reverse(d));
                if !app_times.is_empty() {
                    ui.add_space(8.0);
                    for (i, (app, duration)) in app_times.iter().take(3).enumerate() {
                        let mins = duration.as_secs() / 60;
                        let secs = duration.as_secs() % 60;
                        if i > 0 {
                            ui.add_space(4.0);
                        }
                        ui.label(format!("{}: {:02}:{:02}", app.trim(), mins, secs));
                    }
                }
            });
            ctx.request_repaint();
        });

        // Show notification viewport if open
        if self.notification_open {
            let notification_viewport_id = self.notification_viewport_id;
            ctx.show_viewport_deferred(
                notification_viewport_id,
                eframe::egui::ViewportBuilder::default()
                    .with_title("Notifications")
                    .with_inner_size([300.0, 200.0]),
                move |ctx, _class| {
                    eframe::egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("This is your notifications window!");
                        if ui.button("Close").clicked() {
                            ctx.send_viewport_cmd_to(
                                notification_viewport_id,
                                egui::ViewportCommand::Close,
                            );
                        }
                        // Add more notification-related buttons here
                    });
                },
            );
        }

        if self.show_immediate_viewport {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Immediate Viewport")
                    .with_inner_size([200.0, 100.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello from immediate viewport");
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.show_immediate_viewport = false;
                    }
                },
            );
        }

        // Track notification window state
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
