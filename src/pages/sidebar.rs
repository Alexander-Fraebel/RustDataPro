use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::Ui;

impl DataPro {
    pub fn view_sidebar(&mut self, ui: &mut Ui) {
        self.pick_root_directory.update(ui.ctx());
        if let Some(pathbuf) = self.pick_root_directory.take_picked() {
            // If we change root directory immedately reset all data files, otherwise we have dirty data selections that refers to things which may not exist
            self.data.clear();
            self.root_directory = pathbuf.clone();
        }
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

                        ui.label("Clients Directory");
                        ui.directory_picker(&mut self.pick_root_directory, &self.root_directory);

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // #[cfg(debug_assertions)]
                        // {
                        //     if ui
                        //         .add_sized(
                        //             crate::ui_elements::DEFAULT_LARGE_BUTTOM_DIMS,
                        //             egui::Button::new(
                        //                 egui::RichText::new("DEBUG")
                        //                     .color(ui.visuals().warn_fg_color),
                        //             ),
                        //         )
                        //         .clicked()
                        //     {
                        //         app.display_info.toggle_debug_display();
                        //     }
                        //     ui.add_space(5.0);
                        // }

                        if ui.large_button("Settings").clicked() {
                            self.go_to_settings();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Credits").clicked() {
                            self.go_to_credits();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.large_blue_button("Prepare Session").clicked() {
                            self.go_to_prep_session();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.large_button("Create Client").clicked() {
                            self.go_to_create_client();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Calculate IOA").clicked() {
                            self.go_to_ioa();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Edit KSFs").clicked() {
                            self.go_to_edit_ksf();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Edit Assessments").clicked() {
                            self.go_to_edit_assessments();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.large_button("Shuffle List").clicked() {
                            self.toggle_shuffler_window();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Timers").clicked() {
                            self.toggle_timer_window();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Preference Assessment").clicked() {
                            self.go_to_preference_assessment();
                        }
                        ui.add_space(10.0);
                    });
            });
    }
}
