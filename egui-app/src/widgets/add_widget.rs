use std::sync::Arc;

use arc_swap::ArcSwap;
use background::app::AppState;
use eframe::egui::Ui;
use strum::IntoEnumIterator;

use crate::widgets::WidgetEnum;

#[derive(Debug, PartialEq)]
pub struct AddWidget {}

impl AddWidget {
    pub fn show(&self, ui: &mut Ui, appstate: &Arc<ArcSwap<AppState>>) -> Option<WidgetEnum> {
        ui.label("Add Widget");

        for widget in WidgetEnum::iter() {
            if ui.label(format!("{:?}", widget)).clicked() {
                return Some(widget);
            }
        }
        None
    }
}
