use crate::{
    app::DataPro,
    config::{Config, path_to_config_file},
    quick_error,
    utils::overwrite_file,
};

#[derive(Default)]
pub struct Settings {
    pub config: Config,
    // pub default_root_dir_string: String,
}

impl DataPro {
    pub fn view_settings(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            if ui.button("Manually Set Dark Mode").clicked() {
                ui.ctx().set_visuals(egui::Visuals::dark());
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("UI Scaling");
                if ui
                    .add(
                        egui::DragValue::new(&mut self.settings.config.zoom)
                            .range(1.0..=8.0)
                            .speed(0.1)
                            .fixed_decimals(1),
                    )
                    .lost_focus()
                {
                    ui.ctx().set_pixels_per_point(self.settings.config.zoom);
                    if let Ok(json) = self.settings.config.to_json() {
                        quick_error!(overwrite_file(path_to_config_file(), &json))
                    }
                }
            });
            ui.add_space(10.0);

            ui.label("Default Root Directory");
            if ui
                .text_edit_singleline(&mut self.settings.config.root_dir)
                .lost_focus()
            {
                if let Ok(json) = self.settings.config.to_json() {
                    quick_error!(overwrite_file(path_to_config_file(), &json))
                }
            }
            ui.add_space(10.0);
        });
    }
}
