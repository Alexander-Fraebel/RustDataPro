use crate::{app::DataPro, config::path_to_config_file, quick_error, utils::overwrite_file};

impl DataPro {
    pub fn view_debug_page(&mut self, ui: &mut egui::Ui) {
        let open = &mut self.display_info.debug_open;
        egui::Window::new("Debug Data").open(open).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Client and Session Data");
                    egui::ScrollArea::vertical()
                        .id_salt("debug scroller")
                        .max_height(200.0)
                        .min_scrolled_height(200.0)
                        .max_width(400.0)
                        .show(ui, |ui| {
                            ui.monospace(format!("{}", self.data.to_json().unwrap()));
                        });
                });
                ui.vertical(|ui| {
                    ui.label("Config Data");
                    egui::ScrollArea::vertical()
                        .id_salt("config scroller")
                        .max_height(200.0)
                        .min_scrolled_height(200.0)
                        .max_width(400.0)
                        .show(ui, |ui| {
                            ui.monospace(format!("{}", self.config.to_json().unwrap()));
                        });
                });
            });
            ui.add_space(5.0);

            ui.label("Proportional Font");
            ui.label("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
            ui.add_space(5.0);

            ui.monospace("Monospace Font");
            ui.monospace("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");

            if ui.button("Save Config Data").clicked() {
                quick_error!(overwrite_file(
                    path_to_config_file(),
                    &self.config.to_json().unwrap()
                ))
            }
        });
    }
}
