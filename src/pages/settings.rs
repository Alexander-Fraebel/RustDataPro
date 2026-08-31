use crate::{app::DataPro, config::path_to_config_file, quick_error};
use anyhow::Context;
use std::process::Command;

impl DataPro {
    pub fn view_settings(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
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
                    quick_error!(self.overwrite_config());
                }
            });
            ui.add_space(10.0);

            ui.label("Default Root Directory");
            if ui
                .text_edit_singleline(&mut self.config.root_dir)
                .lost_focus()
            {
                quick_error!(self.overwrite_config());
            }
            ui.add_space(10.0);

            ui.collapsing("Advanced", |ui| {
                ui.label("Config File is Located At");
                if let Ok(path) = path_to_config_file() {
                    ui.label(path.to_string_lossy());
                    ui.horizontal(|ui| {
                        if ui.button("open in Notepad").clicked() {
                            quick_error!(
                                Command::new("notepad")
                                    .arg(path)
                                    .spawn()
                                    .context("unable to open")
                            );
                        }
                        if ui.button("reload Config").clicked() {
                            self.reload_config(ui);
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("UNABLE TO FIND CONFIG");
                        if ui.button("Create").clicked() {
                            quick_error!(self.overwrite_config());
                        }
                    });
                }
            });
            ui.add_space(10.0);
        });
    }
}
