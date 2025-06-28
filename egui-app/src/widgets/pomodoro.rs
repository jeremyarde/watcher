use arc_swap::ArcSwap;
use background::app::AppState;
use eframe::egui;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct PomodoroTimer {
    pub start_at: Option<Instant>,
    pub duration: Duration,
    pub duration_str: String,
    pub show_pomodoro: bool,
    pub label: String,
    pub completed_count: u32,
    pub history: Vec<bool>,
}

impl PomodoroTimer {
    pub fn show(&mut self, ui: &mut egui::Ui, appstate: &Arc<ArcSwap<AppState>>) {
        ui.add_sized([100.0, 0.0], |ui: &mut egui::Ui| {
            ui.horizontal_centered(|ui| {
                ui.add_sized(
                    [100.0, 0.0],
                    egui::TextEdit::singleline(&mut self.label).hint_text("Label"),
                );
                if ui
                    .add_sized(
                        [60.0, 0.0],
                        egui::TextEdit::singleline(&mut self.duration_str).hint_text("mm:ss"),
                    )
                    .lost_focus()
                {
                    if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str) {
                        self.duration = dur;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("🗙").min_size(egui::vec2(24.0, 24.0)))
                        .on_hover_text("Hide Pomodoro")
                        .clicked()
                    {
                        self.show_pomodoro = false;
                    }
                });
            });
            ui.horizontal_centered(|ui| {
                let timer_text = if let Some(_start_at) = self.start_at {
                    let mins = self.remaining_time().as_secs() / 60;
                    let secs = self.remaining_time().as_secs() % 60;
                    format!("{:02}:{:02}", mins, secs)
                } else {
                    "00:00".to_string()
                };
                ui.label(
                    egui::RichText::new(timer_text)
                        .size(24.0)
                        .strong()
                        .monospace()
                        .color(ui.visuals().text_color()),
                );
                if ui.button("Start").clicked() {
                    self.start_at = Some(Instant::now());
                }
                if ui.button("Reset").clicked() {
                    self.start_at = None;
                }
            });
            ui.response()
        });
    }

    pub fn remaining_time(&self) -> Duration {
        let now = Instant::now();
        let duration = now.duration_since(self.start_at.unwrap());
        self.duration - duration
    }

    /// Parse duration from string in the format "mm:ss" or "m" (minutes)
    fn parse_duration(s: &str) -> Option<Duration> {
        let s = s.trim();
        if let Some((min, sec)) = s.split_once(":") {
            let mins: u64 = min.parse().ok()?;
            let secs: u64 = sec.parse().ok()?;
            Some(Duration::from_secs(mins * 60 + secs))
        } else if let Ok(mins) = s.parse::<u64>() {
            Some(Duration::from_secs(mins * 60))
        } else {
            None
        }
    }
}
