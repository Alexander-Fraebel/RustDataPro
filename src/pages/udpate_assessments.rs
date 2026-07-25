use crate::utils::{DataProUiElements, windows_error_dialog};
use crate::{app::DataPro, data::AssessmentsData};
use anyhow::Result;
use indexmap::IndexSet;
use itertools::Itertools;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

pub struct NewAssessments {
    pub assessments: AssessmentsData,
    pub user_inputs: Vec<(String, String)>,
}

impl Default for NewAssessments {
    fn default() -> Self {
        Self {
            assessments: Default::default(),
            user_inputs: Default::default(),
        }
    }
}

impl NewAssessments {
    fn load_assessments_file(&mut self, path: Result<PathBuf>) -> Result<()> {
        match path {
            Ok(pb) => {
                if pb.exists() {
                    self.assessments = AssessmentsData::from_file(&pb)?;
                    for (assessment, conds) in self.assessments.iter() {
                        self.user_inputs
                            .push((assessment.clone(), conds.iter().join(", ")));
                    }
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "client is loaded but assessments.txt does not exist"
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    fn overwrite_or_create_assessments_file(&self, path: Result<PathBuf>) -> Result<()> {
        match path {
            Ok(pb) => {
                if pb.exists() {
                    std::fs::write(pb, &self.assessments.to_json()?)?;
                } else {
                    let mut writer = BufWriter::new(File::create_new(pb)?);
                    writer.write_all(self.assessments.to_json()?.as_bytes())?;
                    writer.flush()?;
                }
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn convert_inputs(&mut self) {
        for (assessment, conditions) in self.user_inputs.iter() {
            if !assessment.trim().is_empty() {
                let conditions_vec: IndexSet<String> = conditions
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.assessments.insert(assessment.clone(), conditions_vec);
            }
        }
    }

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
            if ui.large_button("Load Assessments").clicked() {
                if let Err(e) = app
                    .update_assessments_page
                    .load_assessments_file(app.assessments_path())
                {
                    windows_error_dialog(e)
                }
            }

            ui.add_space(10.0);

            if ui.button("Add Line").clicked() {
                app.update_assessments_page
                    .user_inputs
                    .push((String::new(), String::new()));
            }
            ui.add_space(10.0);

            for (assessment, conditions) in app.update_assessments_page.user_inputs.iter_mut() {
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

            if ui.large_green_button("Save").clicked() {
                // app.new_assessments_page.assessments.clear();
                app.update_assessments_page.convert_inputs();
                if let Err(e) = app
                    .update_assessments_page
                    .overwrite_or_create_assessments_file(app.assessments_path())
                {
                    windows_error_dialog(e)
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
            }
        });
    }
}
