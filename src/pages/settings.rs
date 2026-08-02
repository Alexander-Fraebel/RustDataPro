use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::{RichText, Visuals};

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

            ui.return_button(app, |_| ());
            ui.add_space(10.0);

            if cfg!(debug_assertions) {
                ui.label(
                    RichText::new("⚠ Debug build ⚠")
                        .small()
                        .color(ui.visuals().warn_fg_color),
                );

                egui::ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .content_margin(15.0)
                    .id_salt("scroller")
                    .show(ui, |ui| {
                        ui.monospace(format!("{:#?}", ui.visuals()));
                        ui.add_space(30.0);
                    });
            };
        });
    }
}
