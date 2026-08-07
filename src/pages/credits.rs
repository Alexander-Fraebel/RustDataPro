use crate::{app::DataPro, ui_elements::DataProUiElements};

pub struct Credits {}

impl Credits {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
            ui.add_space(10.0);

            ui.return_button(app, |_| {});
        });
    }
}
