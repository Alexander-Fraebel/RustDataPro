use crate::{app::{ASSESSMENTS_FILE_NAME, DataPro}, configs::DEFAULT_ROOT_DIRECTORY, data::{AssessmentsData, Data}, ui_elements::DataProUiElements, utils::{overwrite_file, windows_error_dialog}};
use egui::{Color32, RichText};
use egui_file_dialog::FileDialog;
use indexmap::IndexSet;
use itertools::Itertools;
use std::path::PathBuf;

fn assessment_scroller(
    app: &mut DataPro,
    ui: &mut egui::Ui,
) -> egui::scroll_area::ScrollAreaOutput<()> {
    if let Some(idx) = app.edit_assessments.deleted_row {
        app.edit_assessments.user_input.remove(idx);
        app.edit_assessments.deleted_row = None;
    }
    ui.add_space(30.0);
    egui::ScrollArea::vertical()
        .min_scrolled_height(400.0)
        .id_salt("assessment_scroller")
        .show(ui, |ui| {
            for (n, (assessment, conditions)) in
                app.edit_assessments.user_input.iter_mut().enumerate()
            {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            (220.0, 18.0),
                            egui::TextEdit::singleline(assessment)
                                .prefix(format!("{}) ", n + 1))
                                .hint_text("Assessment Name"),
                        )
                        .changed()
                    {
                        app.edit_assessments.save_finished = false;
                    }
                    if ui.small_button("delete").clicked() {
                        app.edit_assessments.deleted_row = Some(n)
                    };
                });

                if ui
                    .add(
                        egui::TextEdit::multiline(conditions)
                            .hint_text("Condition1, Condition2, Condition3..."),
                    )
                    .changed()
                {
                    app.edit_assessments.save_finished = false;
                }
                ui.add_space(30.0);
            }
        })
}

fn edit_client_assessments(app: &mut DataPro, ui: &mut egui::Ui) {
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.vertical(|ui| assessment_scroller(app, ui));
        ui.add_space(30.0);
        ui.vertical(|ui| {
            ui.add_space(30.0);
            if ui.button("Add Assessment").clicked() {
                app.edit_assessments
                    .user_input
                    .push((String::new(), String::new()));
            }
            ui.add_space(10.0);

            if ui.large_green_button("Save").clicked() {
                // Update AssessmentsData
                app.data.assessments.clear();
                for (assessment, conditions) in app.edit_assessments.user_input.iter() {
                    if !assessment.trim().is_empty() {
                        let conditions_vec: IndexSet<String> = conditions
                            .split(",")
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        app.data
                            .assessments
                            .insert(assessment.clone(), conditions_vec);
                    }
                }
                app.choose_first_assessment_and_condition();
                if let Err(e) = app.overwrite_assessments() {
                    windows_error_dialog(e)
                } else {
                    app.edit_assessments.save_finished = true;
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
                app.edit_assessments.save_finished = false;
            }

            ui.add_space(10.0);
            if app.edit_assessments.save_finished {
                ui.monospace(
                    RichText::new("Assessments Updated!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    });
}

fn new_assessments(app: &mut DataPro, ui: &mut egui::Ui) {
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.vertical(|ui| assessment_scroller(app, ui));
        ui.add_space(30.0);
        ui.vertical(|ui| {
            ui.add_space(30.0);
            if ui.button("Add Assessment").clicked() {
                app.edit_assessments
                    .user_input
                    .push((String::new(), String::new()));
            }
            ui.add_space(10.0);

            if ui.large_green_button("Save").clicked() {
                let mut temp_assessments_data = AssessmentsData::default();
                for (assessment, conditions) in app.edit_assessments.user_input.iter() {
                    if !assessment.trim().is_empty() {
                        let conditions_vec: IndexSet<String> = conditions
                            .split(",")
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        temp_assessments_data.insert(assessment.clone(), conditions_vec);
                    }
                }

                match temp_assessments_data.to_json() {
                    Ok(json) => {
                        if let Err(e) = overwrite_file(
                            Ok(app
                                .edit_assessments
                                .save_new_path
                                .join(ASSESSMENTS_FILE_NAME)
                                .clone()),
                            &json,
                        ) {
                            windows_error_dialog(e)
                        } else {
                            app.edit_assessments.save_finished = true;
                        }
                    }
                    Err(e) => {
                        windows_error_dialog(e);
                    }
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
                app.edit_assessments.save_finished = false;
            }

            ui.add_space(10.0);
            if app.edit_assessments.save_finished {
                ui.monospace(
                    RichText::new("Assessments Created!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    });
}

#[derive(Debug, Default)]
pub struct EditAssessments {
    pub user_input: Vec<(String, String)>,
    pub save_finished: bool,
    pub deleted_row: Option<usize>,
    pub file_dialog: FileDialog,
    pub save_new_path: PathBuf,
}

impl EditAssessments {
    pub fn prepare(&mut self, data: &Data, path: PathBuf) {
        self.user_input.clear();

        self.save_new_path = path.clone();
        self.file_dialog = FileDialog::new().initial_directory(path.clone());

        // If there is a client loaded rebuild the UI with the client information
        if data.client_loaded() {
            for (assessment, conds) in data.assessments.iter() {
                self.user_input
                    .push((assessment.clone(), conds.iter().join(", ")));
            }
        } else {
            self.file_dialog = FileDialog::new().initial_directory(DEFAULT_ROOT_DIRECTORY.into());
            self.save_new_path = DEFAULT_ROOT_DIRECTORY.into();
        }

        if self.user_input.is_empty() {
            self.user_input.push(Default::default());
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        app.edit_assessments.file_dialog.update(ui.ctx());
        if let Some(pathbuf) = app.edit_assessments.file_dialog.take_picked() {
            app.edit_assessments.save_new_path = pathbuf;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Assessments File");
            app.client_picker(ui);          
            ui.add_space(15.0);

            ui.label("If a client is selected this page will automatically udpate\nthe assessments file for that client. If no client is selected you may\nsave the assessments file created here to the directory below.");
            ui.add_space(10.0);

            ui.add_enabled_ui(!app.data.client_loaded(), |ui| {
                ui.label("Save File To:");
                ui.directory_picker(&mut app.edit_assessments.file_dialog, &app.edit_assessments.save_new_path);
            });
            ui.add_space(10.0);
            if app.data.client_loaded() {
                edit_client_assessments(app, ui)
            } else {
                new_assessments(app, ui)
            }
        });
    }
}
