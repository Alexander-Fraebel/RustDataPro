use egui::warn_if_debug_build;

use crate::{app::DataPro, ui_elements::DataProUiElements};

pub struct DebugPage {}

impl DebugPage {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("sidebar scroller")
            .show(ui, |ui| {
                warn_if_debug_build(ui);
                ui.add_space(10.0);

                ui.return_button(app, |_| {});
                ui.add_space(10.0);

                ui.monospace(format!("{:#?}", app.data));
            });
    }
}
