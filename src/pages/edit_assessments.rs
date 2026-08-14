use crate::{
    app::DataPro,
    data::{AssessmentsData, Conditions, Data},
    ui_elements::DataProUiElements,
    utils::{are_you_sure_dialog, overwrite_file, windows_error_dialog},
};
use egui::{Color32, RichText, TextStyle};
use egui_file_dialog::FileDialog;
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
    ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
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
                                .font(TextStyle::Monospace)
                                .prefix(format!("{}) ", n + 1))
                                .hint_text("Assessment Name"),
                        )
                        .changed()
                    {
                        app.edit_assessments.save_finished = false;
                    }
                    if ui.small_button("delete").clicked() {
                        if are_you_sure_dialog("Delete this Assessment?") {
                            app.edit_assessments.deleted_row = Some(n)
                        }
                    };
                });

                if ui
                    .add(
                        egui::TextEdit::multiline(conditions)
                            .font(TextStyle::Monospace)
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

fn assessments_controller(app: &mut DataPro, ui: &mut egui::Ui) {
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

            if ui.large_green_button("SAVE").clicked() {
                let mut temp_assessments_data = AssessmentsData::default();
                for (assessment, conditions) in app.edit_assessments.user_input.iter() {
                    if !assessment.trim().is_empty() {
                        let conditions_vec: Vec<String> = conditions
                            .split(",")
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        temp_assessments_data
                            .insert(assessment.clone(), Conditions::new(conditions_vec));
                    }
                }

                match temp_assessments_data.to_json() {
                    Ok(json) => {
                        if let Err(e) =
                            overwrite_file(Ok(app.edit_assessments.save_path.clone()), &json)
                        {
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
            ui.add_space(5.0);

            ui.return_button(app, |app| app.edit_assessments.save_finished = false);
            ui.add_space(10.0);

            if app.edit_assessments.save_finished {
                if app.data.client_loaded() {
                    ui.monospace(
                        RichText::new("Assessments Updated!")
                            .heading()
                            .color(Color32::GREEN),
                    );
                } else {
                    ui.monospace(
                        RichText::new("Assessments Created!")
                            .heading()
                            .color(Color32::GREEN),
                    );
                }
            }
        });
    });
}

#[derive(Default)]
pub struct EditAssessments {
    pub user_input: Vec<(String, String)>,
    pub save_finished: bool,
    pub deleted_row: Option<usize>,
    pub file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl EditAssessments {
    pub fn prepare(&mut self, data: &Data, path_to_file: PathBuf) {
        // Reset
        *self = Self::default();

        // Load the path information
        // This will be default information automatically if anything has gone wrong
        self.save_path = path_to_file.clone();
        self.file_dialog = FileDialog::new().initial_directory(path_to_file.clone());

        // If there is a client loaded rebuild the UI with the client information
        if data.client_loaded() {
            for (assessment, conds) in data.assessments.iter() {
                self.user_input
                    .push((assessment.clone(), conds.conditions.iter().join(", ")));
            }
        }
        // Ensure the UI is not empty
        if self.user_input.is_empty() {
            self.user_input.push(Default::default());
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        app.edit_assessments.file_dialog.update(ui.ctx());
        if let Some(pathbuf) = app.edit_assessments.file_dialog.take_picked() {
            app.edit_assessments.save_path = pathbuf;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Assessments File");
            app.client_picker(ui);
            ui.add_space(15.0);

            // ui.label("If a client is selected this page will automatically udpate\nthe assessments file for that client. If no client is selected you may\nsave the assessments file created here to the directory below.");
            // ui.add_space(10.0);

            ui.add_enabled_ui(!app.data.client_loaded(), |ui| {
                ui.label("Save File To:");
                ui.directory_picker(
                    &mut app.edit_assessments.file_dialog,
                    &app.edit_assessments.save_path,
                );
            });
            ui.add_space(10.0);

            assessments_controller(app, ui)
        });
    }
}
