use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([200.0, 20.0])
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
        Box::new(|cc| Ok(Box::new(AppState::new(cc)))),
    )
}

// impl AppState {
//     /// Called once before the first frame.
//     pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
//         // This is also where you can customize the look and feel of egui using
//         // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

//         // Load previous app state (if any).
//         // Note that you must enable the `persistence` feature for this to work.
//         if let Some(storage) = cc.storage {
//             return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
//         }

//         Default::default()
//     }
// }

impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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
            // Display a single line of text with the current session details
            let session_text = if let Ok(sessions) = self.sessions.lock() {
                if let Some(session) = sessions.last() {
                    format!(
                        "App: {} | Window: {}",
                        session.app.trim(),
                        session.window_title.trim()
                    )
                } else {
                    "No session data available".to_string()
                }
            } else {
                "Session data unavailable".to_string()
            };
            ui.label(session_text);
        });
    }
}
