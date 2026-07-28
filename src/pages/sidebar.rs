use crate::{
    app::{DEFAULT_ROOT_DIRECTORY, DataPro, NO_CLIENT},
    ui_elements::DataProUiElements,
};
use egui::{Ui, warn_if_debug_build};
use egui_file_dialog::FileDialog;
use itertools::Itertools;

pub struct Sidebar {}

impl Sidebar {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.pick_root_directory.update(ui.ctx());
        if let Some(pathbuf) = app.pick_root_directory.take_picked() {
            // If we change root directory immedately reset all data files, otherwise we have dirty data selections that refer to things not in the root
            app.data.clear();
            // Then we set the client picker to look there and reset the ksf picker entirely
            app.root_directory = pathbuf.clone();
            app.pick_client_folder = FileDialog::new().initial_directory(pathbuf.clone());
            app.pick_ksf = FileDialog::new()
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

                let ksf_button_name = match app.data.client_loaded() {
                    true => "Edit KSF",
                    false => "New KSF",
                };
                if ui.large_button(ksf_button_name).clicked() {
                    app.edit_ksfs.user_input.clear();

                    // If there is a client loaded rebuild the UI with the client information
                    if app.data.client_loaded() {
                        for (name, ksf) in app.data.ksfs.iter() {
                            let (freq, dura) = ksf.pairs();
                            app.edit_ksfs.user_input.push((
                                name.to_string(),
                                freq.map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                                    .join("\n"),
                                dura.map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                                    .join("\n"),
                            ));
                        }
                    } else {
                        // If there is no client loaded create a UI with a single empty region to start with
                        app.edit_ksfs.file_dialog =
                            FileDialog::new().initial_directory(DEFAULT_ROOT_DIRECTORY.into());
                        app.edit_ksfs.user_input.push(Default::default());
                        app.edit_ksfs.new_ksf_path = DEFAULT_ROOT_DIRECTORY.into();
                    }

                    app.display_info.go_to_new_ksf();
                }
                ui.add_space(5.0);

                let assessment_button_name = match app.data.client_loaded() {
                    true => "Edit Assessments",
                    false => "New Assessments",
                };
                if ui.large_button(assessment_button_name).clicked() {
                    app.edit_assessments.user_input.clear();

                    // If there is a client loaded rebuild the UI with the client information
                    if app.data.client_loaded() {
                        for (assessment, conds) in app.data.assessments.iter() {
                            app.edit_assessments
                                .user_input
                                .push((assessment.clone(), conds.iter().join(", ")));
                        }
                    } else {
                        app.edit_assessments.file_dialog =
                            FileDialog::new().initial_directory(DEFAULT_ROOT_DIRECTORY.into());
                        app.edit_assessments.new_assessments_path = DEFAULT_ROOT_DIRECTORY.into();
                        app.edit_assessments.user_input.push(Default::default());
                    }

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
            });
    }
}
