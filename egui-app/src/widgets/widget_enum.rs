use std::sync::Arc;

use super::application_timer::ApplicationTimer;
use super::pomodoro::PomodoroTimer;
use arc_swap::ArcSwap;
use background::app::AppState;
use eframe::egui::Ui;

pub enum WidgetEnum {
    Pomodoro(PomodoroTimer),
    ApplicationTimer(ApplicationTimer),
}

impl WidgetEnum {
    pub fn show(&mut self, ui: &mut Ui, appstate: &Arc<ArcSwap<AppState>>) {
        match self {
            WidgetEnum::Pomodoro(w) => w.show(ui, appstate),
            WidgetEnum::ApplicationTimer(w) => w.show(ui, &appstate.load()),
        }
    }
}
