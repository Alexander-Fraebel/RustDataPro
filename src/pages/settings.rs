use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::Visuals;

pub struct Settings {}

impl Settings {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Visual Scaling");
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

            ui.add_enabled_ui(false, |ui| {
                if ui
                    .button("Light Mode")
                    .on_disabled_hover_text("not yet available")
                    .clicked()
                {
                    ui.ctx().set_visuals(Visuals::light());
                }
            });
            if ui.button("Dark Mode").clicked() {
                ui.ctx().set_visuals(Visuals::dark());
            }
            ui.add_space(10.0);

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
            }
        });
    }
}
