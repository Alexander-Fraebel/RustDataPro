use crate::app::DataPro;
use crate::utils::{DataProUiElements, windows_error_dialog};
use indexmap::IndexSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveStatus {
    #[default]
    None,
    Saved,
    OverWritten,
}

#[derive(Debug, Default)]
pub struct EditAssessments {
    pub user_inputs: Vec<(String, String)>,
    pub save_status: SaveStatus,
}

impl EditAssessments {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("Update Assessments File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
            ui.add_space(10.0);

            for (assessment, conditions) in app.edit_assessments.user_inputs.iter_mut() {
                ui.horizontal(|ui| {
                    ui.monospace("Assessment");
                    ui.text_edit_singleline(assessment);
                });
                ui.horizontal(|ui| {
                    ui.monospace("Conditions");
                    ui.text_edit_singleline(conditions);
                });
                ui.add_space(15.0);
            }
            ui.add_space(10.0);

            if ui.button("Add Line").clicked() {
                app.edit_assessments
                    .user_inputs
                    .push((String::new(), String::new()));
            }
            ui.add_space(10.0);

            if ui.large_green_button("Save").clicked() {
                // Update AssessmentsData
                app.data.assessments.clear();
                for (assessment, conditions) in app.edit_assessments.user_inputs.iter() {
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
                if let Err(e) = app.overwrite_assessments_file() {
                    windows_error_dialog(e)
                } else {
                    app.edit_assessments.save_status = SaveStatus::Saved;
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
                app.edit_assessments.save_status = SaveStatus::None;
            }

            ui.add_space(10.0);
            match app.edit_assessments.save_status {
                SaveStatus::None => ui.strong(""),
                SaveStatus::Saved => ui.strong("SAVED FILE"),
                SaveStatus::OverWritten => ui.strong("OVER WROTE"),
            }
        });
    }
}
