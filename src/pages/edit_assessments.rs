use crate::{
    app::DataPro,
    data::{Assessment, AssessmentsData, Data},
    quick_error,
    ui_elements::DataProUiElements,
    utils::{are_you_sure_dialog, overwrite_file, windows_error_dialog},
};
use egui::{Color32, RichText, TextStyle};
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
    ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
    ui.add_space(30.0);
    egui::ScrollArea::vertical()
        .min_scrolled_height(400.0)
        .id_salt("assessment_scroller")
        .show(ui, |ui| {
            for (n, assessment) in app.edit_assessments.user_input.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            (220.0, 18.0),
                            egui::TextEdit::singleline(&mut assessment.name)
                                .font(TextStyle::Monospace)
                                .prefix(format!("{}) ", n + 1))
                                .hint_text("Name"),
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
                ui.horizontal(|ui| {
                    ui.label("Current Session:");
                    ui.add(egui::DragValue::new(&mut assessment.session));
                });

                if ui
                    .add(
                        egui::TextEdit::multiline(&mut assessment.conditions)
                            .font(TextStyle::Monospace)
                            .hint_text("Condition1, Condition2, Condition3..."),
                    )
                    .changed()
                {
                    app.edit_assessments.save_finished = false;
                }
                ui.add_space(5.0);
                ui.label("Preferred KSF:");
                ui.text_edit_singleline(&mut assessment.preferred_ksf);
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
                app.edit_assessments.user_input.push(AssessmentMaker::new());
            }
            ui.add_space(10.0);

            if ui.large_green_button("SAVE").clicked() {
                let mut temp_assessments_data = AssessmentsData::default();
                for assessment in app.edit_assessments.user_input.iter() {
                    if !assessment.name.trim().is_empty() {
                        temp_assessments_data
                            .insert(assessment.name.clone(), assessment.into_assessment());
                    }
                }

                match temp_assessments_data.to_json() {
                    Ok(json) => {
                        if let Err(e) =
                            overwrite_file(Ok(app.edit_assessments.save_path.clone()), &json)
                        {
                            windows_error_dialog(e)
                        } else {
                            quick_error!(app.load_assessments());

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

pub struct AssessmentMaker {
    name: String,
    conditions: String,
    session: u32,
    preferred_ksf: String,
}

impl AssessmentMaker {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            conditions: String::new(),
            session: 1,
            preferred_ksf: String::new(),
        }
    }

    pub fn from_assessment(name: &str, assessment: &Assessment) -> Self {
        Self {
            name: name.to_string(),
            conditions: assessment.conditions.iter().join(", "),
            session: assessment.session,
            preferred_ksf: assessment.preferred_ksf.clone(),
        }
    }

    pub fn into_assessment(&self) -> Assessment {
        Assessment {
            session: self.session,
            preferred_ksf: self.preferred_ksf.clone(),
            conditions: IndexSet::from_iter(
                self.conditions
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
            ),
        }
    }
}

#[derive(Default)]
pub struct EditAssessments {
    pub user_input: Vec<AssessmentMaker>,
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
            for (name, assessment) in data.assessments.iter() {
                self.user_input
                    .push(AssessmentMaker::from_assessment(name, assessment))
            }
        }
        // Ensure the UI is not empty
        if self.user_input.is_empty() {
            self.user_input.push(AssessmentMaker::new());
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
