use crate::{app::DataPro, utils::ui_elements::DataProUiElements};
use egui::{RichText, Ui};

impl DataPro {
    pub fn view_sidebar(&mut self, ui: &mut Ui) {
        egui::Panel::left("sidebar")
            .default_size(170.0)
            .min_size(170.0)
            .max_size(170.0)
            .resizable(false)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("sidebar scroller")
                    .show(ui, |ui| {
                        ui.add_space(4.0);

                        ui.horizontal(|ui| {
                            if ui.button("About").clicked() {
                                self.go_to_about();
                            }

                            if ui.button("Settings").clicked() {
                                self.go_to_settings();
                            }
                        });

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if ui.large_blue_button("Prepare Session").clicked() {
                            self.go_to_prep_session();
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if ui.large_button("Edit KSFs").clicked() {
                            self.go_to_edit_ksf();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Edit Assessments").clicked() {
                            self.go_to_edit_assessments();
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if ui.large_button("Create Client").clicked() {
                            self.go_to_create_client();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Calculate IOA").clicked() {
                            self.go_to_ioa();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Collate Files").clicked() {
                            self.go_to_collate();
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if ui.large_button("Shuffle List").clicked() {
                            self.go_to_shuffler();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Timers").clicked() {
                            self.go_to_timers();
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if ui.large_button("Preference Assessments").clicked() {
                            self.go_to_preference_assessment();
                        }
                        ui.add_space(10.0);

                        if cfg!(debug_assertions) {
                            if ui
                                .small_button(
                                    RichText::from("debug").color(ui.visuals().warn_fg_color),
                                )
                                .clicked()
                            {
                                self.go_to_debug();
                            }
                        }
                    });
            });
    }
}
