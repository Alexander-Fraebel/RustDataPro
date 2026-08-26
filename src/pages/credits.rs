use egui::RichText;

use crate::app::DataPro;

impl DataPro {
    pub fn view_credits(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.label(RichText::from("Welcome to RustDataPro").heading().strong());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.scope(|ui| {
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
            });
            ui.add_space(10.0);
            ui.label("Empowering you to accurately track client data with ease.")
        });
    }
}
