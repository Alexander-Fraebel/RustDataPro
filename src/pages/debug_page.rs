use crate::{app::DataPro, config::path_to_config_file, quick_error, utils::overwrite_file};

const EXAMPLE_TEXT: &'static str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\n\nMen act upon the world, and change it, and are changed in turn by the consequences of their actions. Certain processes, which the human organism shares with other species, alter behavior so that it achieves a safer and more useful interchange with a particular environment. When selfropriate behavior has been established, its consequences work through similar processes to keep it in force. If by chance the environment changes, old forms of behavior disselfear, while new consequences build new forms.";

impl DataPro {
    pub fn view_debug_page(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.collapsing("Client and Session Data", |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("debug scroller")
                    .max_height(200.0)
                    .min_scrolled_height(200.0)
                    .max_width(500.0)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", self.data.to_json().unwrap()));
                    });
            });
            ui.add_space(5.0);
            ui.collapsing("Config Data", |ui| {
                if ui.button("Save Config Data").clicked() {
                    quick_error!(overwrite_file(
                        path_to_config_file(),
                        &self.config.to_json().unwrap()
                    ))
                }
                egui::ScrollArea::vertical()
                    .id_salt("config scroller")
                    .max_height(200.0)
                    .min_scrolled_height(200.0)
                    .max_width(500.0)
                    .show(ui, |ui| {
                        ui.monospace(format!("{}", self.config.to_json().unwrap()));
                    });
            });
            ui.add_space(5.0);

            ui.collapsing("Font Examples", |ui| {
                ui.label("Proportional");
                ui.label(EXAMPLE_TEXT);
                ui.add_space(15.0);

                ui.monospace("Monospace");
                ui.monospace(EXAMPLE_TEXT);
            });
            ui.add_space(5.0);
        });
    }
}
