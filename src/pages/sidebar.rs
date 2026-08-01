use crate::{
    app::{DEFAULT_ROOT_DIRECTORY, DataPro, NO_CLIENT},
    ui_elements::DataProUiElements,
};
use egui::{Ui, warn_if_debug_build};

pub struct Sidebar {}

impl Sidebar {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.pick_root_directory.update(ui.ctx());
        if let Some(pathbuf) = app.pick_root_directory.take_picked() {
            // If we change root directory immedately reset all data files, otherwise we have dirty data selections that refer to things not in the root
            app.data.clear();
            // Then we set the client picker to look there and reset the ksf picker entirely
            app.root_directory = pathbuf.clone();
        }
        egui::Panel::left("welcome_panel")
            .default_size(200.0)
            .min_size(200.0)
            .max_size(200.0)
            .resizable(false)
            .show(ui, |ui| {
                warn_if_debug_build(ui);
                ui.strong("Welcome to RutgersDataPro!");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("Clients Directory");
                ui.directory_picker(&mut app.pick_root_directory, &app.root_directory);

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                if ui.large_button("Create Client").clicked() {
                    app.display_info.go_to_new_client();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                if ui
                    .large_button("Calculate IOA")
                    .on_disabled_hover_text(NO_CLIENT)
                    .clicked()
                {
                    app.display_info.go_to_ioa();
                }
                ui.add_space(5.0);

                if ui.large_button("KSF").clicked() {
                    app.edit_ksfs.prepare(
                        &app.data,
                        app.path_to_ksf_data()
                            .unwrap_or(DEFAULT_ROOT_DIRECTORY.into()),
                    );
                    app.display_info.go_to_new_ksf();
                }
                ui.add_space(5.0);

                if ui.large_button("Assessments").clicked() {
                    app.edit_assessments.prepare(
                        &app.data,
                        app.path_to_assessments()
                            .unwrap_or(DEFAULT_ROOT_DIRECTORY.into()),
                    );
                    app.display_info.go_to_new_assessments();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                if ui.large_button("Randomness").clicked() {
                    app.display_info.toggle_random_display();
                }
                ui.add_space(5.0);

                if ui.large_button("Timers").clicked() {
                    app.display_info.toggle_timer_display();
                }
                ui.add_space(5.0);

                if ui.large_button("Settings").clicked() {
                    app.display_info.go_to_settings();
                }
                ui.add_space(5.0);
            });
    }
}
