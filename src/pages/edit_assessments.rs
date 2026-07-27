use crate::app::DataPro;
use crate::utils::{DataProUiElements, windows_error_dialog};
use egui::{Color32, RichText};
use indexmap::IndexSet;

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

#[derive(Debug, Default)]
pub struct EditAssessments {
    pub user_input: Vec<(String, String)>,
    pub save_finished: bool,
    pub deleted_row: Option<usize>,
}

impl EditAssessments {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("Edit Assessments File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
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
        });
    }
}
