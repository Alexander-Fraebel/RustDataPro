use crate::{app::DataPro, config::Config, ui_elements::DataProUiElements};
use std::fs::File;

pub struct Settings {}

impl Settings {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("UI Scaling");
                if ui
                    .add(
                        egui::DragValue::new(&mut app.display_info.zoom)
                            .range(1.0..=2.0)
                            .speed(0.1)
                            .fixed_decimals(1),
                    )
                    .lost_focus()
                {
                    ui.ctx().set_pixels_per_point(app.display_info.zoom);
                }
            });
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(10.0);

            if ui.large_blue_button("Create Config File").clicked() {
                let config_file = File::create("config.json").unwrap();
                let mut writer = std::io::BufWriter::new(config_file);
                std::io::Write::write_all(
                    &mut writer,
                    Config::default().to_json().unwrap().as_bytes(),
                )
                .unwrap();
                std::io::Write::flush(&mut writer).unwrap();
            }
            ui.add_space(10.0);

            ui.separator();
            ui.add_space(10.0);

            ui.return_button(app, |_| ());
        });
    }
}
