use arc_swap::ArcSwap;
use background::app::AppState;
use eframe::egui;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Default)]
pub struct PomodoroTimer {
    pub start_at: Option<Instant>,
    pub duration: Duration,
    pub duration_str: String,
    pub show_pomodoro: bool,
    pub label: String,
    pub completed_count: u32,
    pub history: Vec<bool>,
    pub is_running: bool,
}

impl PomodoroTimer {
    pub fn hide_widget(&mut self) {
        self.show_pomodoro = false;
    }
    pub fn show_widget(&mut self) {
        self.show_pomodoro = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, appstate: &Arc<ArcSwap<AppState>>) {
        let available_width = ui.available_width();

        if !self.show_pomodoro {
            if ui.button("Show").clicked() {
                self.show_pomodoro = true;
            }
            return;
        }

        // Determine if we're in a very narrow viewport
        let is_narrow = available_width < 200.0;

        // Main container with rounded corners and subtle background
        egui::Frame::group(ui.style())
            .fill(ui.visuals().panel_fill)
            .corner_radius(8.0)
            .inner_margin(if is_narrow {
                egui::Margin::symmetric(8, 6)
            } else {
                egui::Margin::symmetric(12, 8)
            })
            .show(ui, |ui| {
                if is_narrow {
                    // Compact layout for narrow viewports
                    ui.vertical(|ui| {
                        // Header with label and close button
                        ui.horizontal(|ui| {
                            let label_width = (available_width * 0.6).max(80.0);
                            ui.add_sized(
                                egui::vec2(label_width, 20.0),
                                egui::TextEdit::singleline(&mut self.label).hint_text("Task"),
                            );

                            ui.add_space(4.0);

                            if ui
                                .add_sized(egui::vec2(16.0, 20.0), egui::Button::new("×"))
                                .on_hover_text("Remove Pomodoro")
                                .clicked()
                            {
                                self.show_pomodoro = false;
                            }
                        });

                        // Timer display/input
                        ui.horizontal(|ui| {
                            if self.is_running {
                                let remaining = self.remaining_time();
                                let mins = remaining.as_secs() / 60;
                                let secs = remaining.as_secs() % 60;
                                let timer_text = format!("{:02}:{:02}", mins, secs);

                                ui.label(
                                    egui::RichText::new(timer_text)
                                        .size(18.0)
                                        .monospace()
                                        .color(egui::Color32::from_rgb(100, 200, 100)),
                                );
                            } else {
                                let duration_width = (available_width * 0.3).max(40.0);
                                let response = ui.add_sized(
                                    egui::vec2(duration_width, 20.0),
                                    egui::TextEdit::singleline(&mut self.duration_str)
                                        .hint_text("mm:ss"),
                                );

                                if response.changed() {
                                    if let Some(dur) =
                                        PomodoroTimer::parse_duration(&self.duration_str)
                                    {
                                        self.duration = dur;
                                    }
                                }
                            }
                        });

                        // Compact controls
                        ui.horizontal(|ui| {
                            let (button_text, button_color) = if self.is_running {
                                ("⏸", egui::Color32::from_rgb(255, 165, 0))
                            } else {
                                ("▶", egui::Color32::from_rgb(100, 200, 100))
                            };

                            if ui
                                .add_sized(
                                    egui::vec2(24.0, 20.0),
                                    egui::Button::new(
                                        egui::RichText::new(button_text).color(button_color),
                                    ),
                                )
                                .clicked()
                            {
                                if self.is_running {
                                    let remaining = self.remaining_time();
                                    self.duration = remaining;
                                    self.start_at = None;
                                    self.is_running = false;
                                } else {
                                    self.start_at = Some(Instant::now());
                                    self.is_running = true;
                                }
                            }

                            ui.add_space(2.0);

                            if ui
                                .add_sized(egui::vec2(24.0, 20.0), egui::Button::new("⟲"))
                                .clicked()
                            {
                                self.start_at = None;
                                self.is_running = false;
                                if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str)
                                {
                                    self.duration = dur;
                                }
                            }

                            ui.add_space(2.0);

                            if ui
                                .add_sized(egui::vec2(24.0, 20.0), egui::Button::new("✔"))
                                .clicked()
                            {
                                self.completed_count += 1;
                                self.history.push(true);
                                self.start_at = None;
                                self.is_running = false;
                                if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str)
                                {
                                    self.duration = dur;
                                }
                            }
                        });
                    });
                } else {
                    // Standard layout for wider viewports
                    // Header row with label and close button
                    ui.horizontal(|ui| {
                        // Make label input flexible but with reasonable bounds
                        let label_width = (available_width * 0.4).max(60.0).min(120.0);
                        ui.add_sized(
                            egui::vec2(label_width, 24.0),
                            egui::TextEdit::singleline(&mut self.label).hint_text("Task"),
                        );

                        ui.add_space(8.0);

                        if self.is_running {
                            // Show remaining time as read-only display
                            let remaining = self.remaining_time();
                            let mins = remaining.as_secs() / 60;
                            let secs = remaining.as_secs() % 60;
                            let timer_text = format!("{:02}:{:02}", mins, secs);

                            ui.label(
                                egui::RichText::new(timer_text)
                                    .size(20.0)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            );
                        } else {
                            // Show duration as editable input with flexible width
                            let duration_width = (available_width * 0.25).max(50.0).min(80.0);
                            let response = ui.add_sized(
                                egui::vec2(duration_width, 24.0),
                                egui::TextEdit::singleline(&mut self.duration_str)
                                    .hint_text("mm:ss"),
                            );

                            // Parse duration whenever the text changes
                            if response.changed() {
                                if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str)
                                {
                                    self.duration = dur;
                                }
                            }
                        }

                        ui.add_space(8.0);

                        // Make close button smaller and more compact
                        if ui
                            .add_sized(egui::vec2(20.0, 24.0), egui::Button::new("×"))
                            .on_hover_text("Remove Pomodoro")
                            .clicked()
                        {
                            self.show_pomodoro = false;
                        }
                    });
                    ui.add_space(8.0);

                    // Controls section - make buttons more compact
                    ui.horizontal(|ui| {
                        let (button_text, button_color) = if self.is_running {
                            ("⏸", egui::Color32::from_rgb(255, 165, 0))
                        } else {
                            ("▶", egui::Color32::from_rgb(100, 200, 100))
                        };

                        if ui
                            .add_sized(
                                egui::vec2(28.0, 24.0),
                                egui::Button::new(
                                    egui::RichText::new(button_text).color(button_color),
                                ),
                            )
                            .clicked()
                        {
                            if self.is_running {
                                // Pause - store remaining time
                                let remaining = self.remaining_time();
                                self.duration = remaining;
                                self.start_at = None;
                                self.is_running = false;
                            } else {
                                // Start
                                self.start_at = Some(Instant::now());
                                self.is_running = true;
                            }
                        }

                        ui.add_space(4.0);

                        if ui
                            .add_sized(egui::vec2(28.0, 24.0), egui::Button::new("⟲"))
                            .clicked()
                        {
                            self.start_at = None;
                            self.is_running = false;
                            // Reset to original duration
                            if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str) {
                                self.duration = dur;
                            }
                        }

                        ui.add_space(4.0);

                        if ui
                            .add_sized(egui::vec2(28.0, 24.0), egui::Button::new("✔"))
                            .clicked()
                        {
                            self.completed_count += 1;
                            self.history.push(true);
                            self.start_at = None;
                            self.is_running = false;
                            // Reset to original duration
                            if let Some(dur) = PomodoroTimer::parse_duration(&self.duration_str) {
                                self.duration = dur;
                            }
                        }
                    });
                }
            });
    }

    // fn get_timer_info(&self) -> (String, bool, f32) {
    //     let is_running = self.start_at.is_some();

    //     if is_running {
    //         let remaining = self.remaining_time();
    //         let total_secs = self.duration.as_secs();
    //         let remaining_secs = remaining.as_secs();

    //         if remaining_secs <= 0 {
    //             // Timer finished
    //             return ("00:00".to_string(), false, 1.0);
    //         }

    //         let mins = remaining_secs / 60;
    //         let secs = remaining_secs % 60;
    //         let timer_text = format!("{:02}:{:02}", mins, secs);
    //         let progress = 1.0 - (remaining_secs as f32 / total_secs as f32);

    //         (timer_text, true, progress)
    //     } else {
    //         let total_secs = self.duration.as_secs();
    //         let mins = total_secs / 60;
    //         let secs = total_secs % 60;
    //         let timer_text = format!("{:02}:{:02}", mins, secs);

    //         (timer_text, false, 0.0)
    //     }
    // }

    pub fn remaining_time(&self) -> Duration {
        if let Some(start_at) = self.start_at {
            let now = Instant::now();
            let elapsed = now.duration_since(start_at);
            if elapsed >= self.duration {
                Duration::ZERO
            } else {
                self.duration - elapsed
            }
        } else {
            // When not running, return the current duration setting
            self.duration
        }
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
