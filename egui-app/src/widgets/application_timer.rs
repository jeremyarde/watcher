use background::app::AppState;
use eframe::egui;
use std::time::Duration;

pub struct ApplicationTimer {
    pub app: String,
    pub duration: Duration,
}

impl ApplicationTimer {
    pub fn show(&self, ui: &mut egui::Ui, appstate: &AppState) {
        let active_session = appstate.active_session.clone();
        let stats = appstate.stats.clone();
        ui.vertical_centered_justified(|ui| {
            if let Some(session) = active_session {
                let now = std::time::Instant::now();
                let duration = now.duration_since(session.start_at);
                let secs = duration.as_secs();
                let mins = secs / 60;
                let secs = secs % 60;
                ui.label(egui::RichText::new(format!("{}", session.app.trim())).strong());
                // ui.add_space(4.0);
                if !session.window_title.trim().is_empty() {
                    ui.label(egui::RichText::new(session.window_title.trim()).italics());
                }
                // ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("⏱ {:02}:{:02}", mins, secs)).monospace());
            } else {
                ui.label("No session");
            }

            // Top 3 apps summary, compact
            let mut app_times: Vec<_> = stats.total_time_per_app.iter().collect();
            app_times.sort_by_key(|&(_, &d)| std::cmp::Reverse(d));
            if !app_times.is_empty() {
                ui.add_space(4.0);
                for (i, (app, duration)) in app_times.iter().take(2).enumerate() {
                    let mins = duration.as_secs() / 60;
                    let secs = duration.as_secs() % 60;
                    if i > 0 {
                        ui.add_space(4.0);
                    }
                    ui.label(format!("{}: {:02}:{:02}", app.trim(), mins, secs));
                }
            }
        });
    }
}
