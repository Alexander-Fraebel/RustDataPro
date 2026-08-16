use crate::{
    app::DataPro,
    config::{Config, path_to_config_file},
    ui_elements::DataProUiElements,
    utils::{overwrite_file, windows_error_dialog},
};
use anyhow::Result;

#[derive(Default)]
pub struct Settings {
    pub config: Config,
    pub default_root_dir_string: String,
}

impl Settings {
    pub fn update_config_file(&mut self) -> Result<()> {
        self.config.root_dir = self.default_root_dir_string.clone().into();
        overwrite_file(path_to_config_file(), &self.config.to_json()?)
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
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
                        egui::DragValue::new(&mut app.settings.config.zoom)
                            .range(1.0..=8.0)
                            .speed(0.1)
                            .fixed_decimals(1),
                    )
                    .lost_focus()
                {
                    ui.ctx().set_pixels_per_point(app.settings.config.zoom);
                }
            });
            ui.add_space(10.0);

            ui.label("Default Root Directory");
            ui.text_edit_singleline(&mut app.settings.default_root_dir_string);

            if ui.large_blue_button("Create Config File").clicked() {
                app.settings
                    .update_config_file()
                    .unwrap_or_else(|e| windows_error_dialog(e))
            }
            ui.label(format!(
                "saves to:\n{}",
                path_to_config_file()
                    .map_or("NOT FOUND".into(), |pb| pb.to_string_lossy().to_string()),
            ));
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(10.0);

            ui.return_button(app, |app| {
                app.settings
                    .update_config_file()
                    .unwrap_or_else(|e| windows_error_dialog(e))
            });
        });
    }
}
