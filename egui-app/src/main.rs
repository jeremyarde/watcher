use arc_swap::ArcSwap;
use background::app::AppState;
use background::pomodoro::PomodoroState;
use eframe::egui::viewport::ViewportId;
use eframe::egui::{self, Widget};
use log::info;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::widgets::{AddWidget, WidgetEnum};
mod widgets;

// #[derive(Deserialize, Serialize)]
// #[derive(Debug)]
pub struct MyApp {
    appstate: Arc<ArcSwap<AppState>>,
    notification_open: bool,
    notification_viewport_id: ViewportId,
    show_immediate_viewport: bool,
    show_deferred_viewport: Arc<AtomicBool>,
    minimized: bool,
    maximized_position: egui::Pos2,
    minimized_position: egui::Pos2,
    inner_size: egui::Vec2,
    maximized_size: egui::Vec2,
    minimized_size: egui::Vec2,
    top_panel_hovered: bool,
    // Visibility toggles for widgets
    show_title_bar: bool,
    show_main_content: bool,
    show_immediate_window: bool,
    show_deferred_window: bool,
    show_pomodoro: bool,
    widgets: Vec<WidgetEnum>,
}

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
            // .with_inner_size([400.0, 50.0])
            // .with_titlebar_buttons_shown(true)
            // .with_title_shown(false)
            .with_taskbar(false)
            .with_always_on_top()
            .with_decorations(false)
            .with_transparent(true)
            // .with_position([1000.0, 100.0])
            .with_inner_size([400.0, 100.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc, appstate)))),
    )
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
            show_deferred_viewport: Arc::new(AtomicBool::new(false)),
            minimized: false,
            maximized_position: egui::Pos2::new(1000.0, 100.0),
            minimized_position: egui::Pos2::new(10.0, 10.0),
            inner_size: egui::Vec2::new(400.0, 50.0),
            maximized_size: egui::Vec2::new(400.0, 50.0),
            minimized_size: egui::Vec2::new(400.0, 50.0),
            top_panel_hovered: false,
            show_title_bar: true,
            show_main_content: true,
            show_immediate_window: true,
            show_deferred_window: true,
            show_pomodoro: true,
            widgets: vec![
                WidgetEnum::Pomodoro(crate::widgets::pomodoro::PomodoroTimer {
                    duration: Duration::from_secs(25 * 60),
                    duration_str: "25:00".to_string(),
                    start_at: None,
                    show_pomodoro: true,
                    label: String::from("Task"),
                    completed_count: 0,
                    history: vec![],
                    is_running: false,
                }),
                WidgetEnum::ApplicationTimer(crate::widgets::application_timer::ApplicationTimer {
                    app: "Application".to_string(),
                    duration: Duration::from_secs(25 * 60),
                }),
            ],
        }
    }

    fn show_file_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let is_web = cfg!(target_arch = "wasm32");
        if !is_web {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                egui::widgets::global_theme_preference_buttons(ui);
            });
        }
    }

    fn show_hide_button(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        just_moved_window: &mut bool,
    ) {
        if ui.button("Hide").clicked() {
            ctx.input(|i| {
                if let Some(rect) = i.viewport().outer_rect {
                    self.maximized_position = rect.min;
                    self.maximized_size = rect.size();
                }
            });
            self.minimized = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.minimized_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                self.minimized_position,
            ));
            *just_moved_window = true;
        }
    }

    fn show_drag_area(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let available = ui.available_size_before_wrap();
        if available.y > 0.0 {
            let (_, rect) = ui.allocate_space(egui::vec2(32.0, 32.0));
            let response = ui
                .interact(
                    rect,
                    ui.id().with("drag_area"),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_cursor(egui::CursorIcon::Grab);
            if response.dragged() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if response.drag_stopped() {
                ctx.input(|i| {
                    if let Some(rect) = i.viewport().outer_rect {
                        self.maximized_position = rect.min;
                        self.maximized_size = rect.size();
                    }
                    log::info!(
                        "Maximized drag stopped: pos={:?}, size={:?}",
                        self.maximized_position,
                        self.maximized_size
                    );
                });
            }
        }
    }
}

impl eframe::App for MyApp {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Track if we just programmatically moved the window
        let mut just_moved_window = false;

        // Always track the current position and size
        let (current_pos, current_size) = ctx.input(|i| {
            if let Some(rect) = i.viewport().outer_rect {
                (rect.min, rect.size())
            } else {
                (egui::Pos2::new(0.0, 0.0), egui::Vec2::new(0.0, 0.0))
            }
        });
        if self.minimized {
            // Update minimized position and size if window moved or resized
            if current_pos != self.minimized_position {
                self.minimized_position = current_pos;
            }
            if current_size != self.minimized_size {
                self.minimized_size = current_size;
            }
        } else {
            // Update maximized position and size if window moved or resized
            if current_pos != self.maximized_position {
                self.maximized_position = current_pos;
            }
            if current_size != self.maximized_size {
                self.maximized_size = current_size;
            }
        }

        // Unhide menu if any panels are hidden
        if !self.show_title_bar
            || !self.show_main_content
            || !self.show_immediate_window
            || !self.show_deferred_window
            || !self.show_pomodoro
        {
            egui::TopBottomPanel::bottom("unhide_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Show hidden widgets:");
                    if !self.show_title_bar && ui.button("Title Bar").clicked() {
                        self.show_title_bar = true;
                    }
                    if !self.show_main_content && ui.button("Main Content").clicked() {
                        self.show_main_content = true;
                    }
                    if !self.show_pomodoro && ui.button("Pomodoro Timer").clicked() {
                        self.show_pomodoro = true;
                    }
                    if !self.show_immediate_window && ui.button("Immediate Window").clicked() {
                        self.show_immediate_window = true;
                    }
                    if !self.show_deferred_window && ui.button("Deferred Window").clicked() {
                        self.show_deferred_window = true;
                    }
                });
            });
        }

        if self.show_title_bar {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        // Drag handle with background and border
                        let drag_area = ui
                            .add_sized(
                                [60.0, 24.0],
                                egui::Label::new("⠿ Move")
                                    .sense(egui::Sense::click_and_drag())
                                    .wrap(),
                            )
                            .on_hover_text("Drag window")
                            .highlight();
                        if drag_area.dragged() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        ui.separator();
                        self.show_file_button(ui, ctx);
                        ui.separator();
                        self.show_hide_button(ui, ctx, &mut just_moved_window);
                        ui.separator();
                        if ui.button("🗙").on_hover_text("Hide this top bar").clicked() {
                            self.show_title_bar = false;
                        }
                    });
                });
                // Subtle separator below top bar
                ui.add(egui::Separator::default());
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            puffin::profile_scope!("central_panel");
            let mut widgets_to_remove = Vec::new();

            // Determine layout based on available width
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let use_horizontal = available_width > 600.0; // Threshold for switching to horizontal

            // Calculate widget count before the loops to avoid borrow checker issues
            let widget_count = self.widgets.len() as f32;

            if use_horizontal {
                // Horizontal layout for wider viewports
                ui.horizontal_wrapped(|ui| {
                    for (index, widget) in self.widgets.iter_mut().enumerate() {
                        puffin::profile_scope!("show_widget");
                        // Calculate widget size to fill available space with better bounds checking
                        let widget_width = ((available_width - 40.0) / widget_count.max(1.0))
                            .max(150.0)
                            .min(available_width - 40.0); // Better minimum width
                        let widget_height = (available_height - 40.0).max(80.0); // Better minimum height

                        egui::Frame::group(ui.style())
                            .fill(ui.visuals().extreme_bg_color)
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(4))
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(widget_width, widget_height));
                                ui.vertical(|ui| {
                                    widget.show(ui, &self.appstate);
                                    ui.add_space(4.0);
                                    if ui.button("🗑").on_hover_text("Delete widget").clicked() {
                                        widgets_to_remove.push(index);
                                    }
                                });
                            });
                    }
                });
            } else {
                // Single column layout for narrow viewports
                ui.vertical(|ui| {
                    for (index, widget) in self.widgets.iter_mut().enumerate() {
                        puffin::profile_scope!("show_widget");
                        // Calculate widget size to fill available space with better bounds checking
                        let widget_height =
                            ((available_height - 60.0) / widget_count.max(1.0)).max(80.0); // Better minimum height
                        let widget_width = (available_width - 20.0).max(150.0); // Better minimum width

                        egui::Frame::group(ui.style())
                            .fill(ui.visuals().extreme_bg_color)
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(4))
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(widget_width, widget_height));
                                ui.horizontal(|ui| {
                                    widget.show(ui, &self.appstate);
                                    ui.add_space(4.0);
                                    if ui.button("🗑").on_hover_text("Delete widget").clicked() {
                                        widgets_to_remove.push(index);
                                    }
                                });
                            });
                    }
                });
            }

            // Remove widgets that were marked for deletion (in reverse order to maintain indices)
            for &index in widgets_to_remove.iter().rev() {
                if index < self.widgets.len() {
                    self.widgets.remove(index);
                }
            }

            // Add widget button with dropdown
            ui.horizontal(|ui| {
                ui.menu_button("➕ Add Widget", |ui| {
                    if ui.button("Pomodoro Timer").clicked() {
                        self.widgets.push(WidgetEnum::Pomodoro(
                            crate::widgets::pomodoro::PomodoroTimer {
                                duration: Duration::from_secs(25 * 60),
                                duration_str: "25:00".to_string(),
                                start_at: None,
                                show_pomodoro: true,
                                label: String::from("New Task"),
                                completed_count: 0,
                                history: vec![],
                                is_running: false,
                            },
                        ));
                        ui.close_menu();
                    }
                    if ui.button("Application Timer").clicked() {
                        self.widgets.push(WidgetEnum::ApplicationTimer(
                            crate::widgets::application_timer::ApplicationTimer {
                                app: "Application".to_string(),
                                duration: Duration::from_secs(0),
                            },
                        ));
                        ui.close_menu();
                    }
                });
            });

            ctx.request_repaint();
        });
    }
}

fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("127.0.0.1:8585") {
        Ok(puffin_server) => {
            log::info!("Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            // Check if puffin_viewer is already running
            let viewer_running = TcpStream::connect("127.0.0.1:8585").is_ok();

            if !viewer_running {
                std::process::Command::new("puffin_viewer")
                    .arg("--url")
                    .arg("127.0.0.1:8585")
                    .spawn()
                    .ok();
            } else {
                log::info!("puffin_viewer already running, not spawning a new one.");
            }

            #[expect(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            log::error!("Failed to start puffin server: {err}");
        }
    };
}
