use anyhow::Context;

use crate::{app::DataPro, config::path_to_config_file, quick_error, utils::overwrite_file};
use std::process::Command;

impl DataPro {
    pub fn view_settings(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);

            ui.label("Config File is Located At");
            if let Ok(path) = path_to_config_file() {
                ui.horizontal(|ui| {
                    ui.label(path.to_string_lossy());
                    if ui.button("open in Notepad").clicked() {
                        quick_error!(
                            Command::new("notepad")
                                .arg(path)
                                .spawn()
                                .context("unable to open")
                        );
                    }
                });
            } else {
                ui.label("UNABLE TO FIND CONFIG FILE PATH");
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("UI Scaling");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.config.zoom)
                            .range(1.0..=8.0)
                            .speed(0.1)
                            .fixed_decimals(1),
                    )
                    .lost_focus()
                {
                    ui.ctx().set_pixels_per_point(self.config.zoom);
                    if let Ok(json) = self.config.to_json() {
                        quick_error!(overwrite_file(path_to_config_file(), &json))
                    }
                }
            });
            ui.add_space(10.0);

            ui.label("Default Root Directory");
            if ui
                .text_edit_singleline(&mut self.config.root_dir)
                .lost_focus()
            {
                if let Ok(json) = self.config.to_json() {
                    quick_error!(overwrite_file(path_to_config_file(), &json))
                }
            }
            ui.add_space(10.0);
        });
    }
}
