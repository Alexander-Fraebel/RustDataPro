use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::{Ui, warn_if_debug_build};

pub struct Sidebar {}

impl Sidebar {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.pick_root_directory.update(ui.ctx());
        if let Some(pathbuf) = app.pick_root_directory.take_picked() {
            // If we change root directory immedately reset all data files, otherwise we have dirty data selections that refers to things which may not exist
            app.data.clear();
            app.root_directory = pathbuf.clone();
        }
        egui::Panel::left("welcome_panel")
            .default_size(170.0)
            .min_size(170.0)
            .max_size(170.0)
            .resizable(false)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("sidebar scroller")
                    .show(ui, |ui| {
                        warn_if_debug_build(ui);
                        ui.strong("Welcome to RustDataPro!");
                        ui.add_space(4.0);

                        ui.label("Clients Directory");
                        ui.directory_picker(&mut app.pick_root_directory, &app.root_directory);

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
                            app.display_info.go_to_settings();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Credits").clicked() {
                            app.display_info.go_to_credits();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.large_button("Create Client").clicked() {
                            app.display_info.go_to_new_client();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Calculate IOA").clicked() {
                            app.display_info.go_to_ioa();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("KSF").clicked() {
                            app.display_info.go_to_new_ksf();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Assessments").clicked() {
                            app.display_info.go_to_new_assessments();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.large_button("Shuffle List").clicked() {
                            app.display_info.toggle_random_display();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Timers").clicked() {
                            app.display_info.toggle_timer_display();
                        }
                        ui.add_space(4.0);

                        if ui.large_button("Preference Assessment").clicked() {
                            app.display_info.go_to_preference_assessment();
                        }
                        ui.add_space(8.0);
                    });
            });
    }
}
