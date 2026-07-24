use crate::app::ASSESSMENTS_FILE_NAME;
use crate::data::Data;
use crate::utils::DataProUiElements;
use crate::{app::DataPro, data::AssessmentsData};
use anyhow::Result;
use indexmap::IndexSet;
use std::path::Path;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

pub struct NewAssessments {
    pub assessments: AssessmentsData,
    pub user_inputs: Vec<(String, String)>,
    pub error: String,
}

impl Default for NewAssessments {
    fn default() -> Self {
        Self {
            assessments: Default::default(),
            user_inputs: vec![(String::new(), String::new())],
            error: Default::default(),
        }
    }
}

impl NewAssessments {
    fn save_file_to_path(&mut self, data: &Data, root_directory: &PathBuf) -> Result<()> {
        let p = Path::new(root_directory)
            .join(&format!("{}", data.client.id))
            .join(ASSESSMENTS_FILE_NAME);
        let mut writer = BufWriter::new(File::create_new(p)?);
        writer.write_all(self.assessments.to_json()?.as_bytes())?;
        writer.flush()?;

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
                ui.heading("New Assessments File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
            ui.add_space(10.0);

            if ui.button("Add Line").clicked() {
                app.new_assessments_page
                    .user_inputs
                    .push((String::new(), String::new()));
            }
            ui.add_space(10.0);

            for (assessment, conditions) in app.new_assessments_page.user_inputs.iter_mut() {
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
                app.new_assessments_page.assessments.clear();
                app.new_assessments_page.convert_inputs();
                match app
                    .new_assessments_page
                    .save_file_to_path(&app.data, &app.root_directory)
                {
                    Ok(_) => app.new_assessments_page.error.clear(),
                    Err(e) => app.new_assessments_page.error = e.to_string(),
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.display_info.go_to_prep_session();
            }

            ui.strong(app.new_assessments_page.error.to_string());
        });
    }
}
