use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::Ui;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum PaType {
    #[default]
    None,
    PairedChoice,
}

impl Display for PaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaType::None => write!(f, "None"),
            PaType::PairedChoice => write!(f, "Paired Choice"),
        }
    }
}

#[derive(Default)]
pub struct PreferenceAssessment {
    pub patype: PaType,
    pub conditions: Vec<String>,
}

impl PreferenceAssessment {
    pub fn all_pairs(&self) -> Vec<(String, String)> {
        let a = self.conditions.clone();
        let b = self.conditions.clone();
        a.into_iter().cartesian_product(b.into_iter()).collect()
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ComboBox::from_id_salt("preference_assessment")
                .selected_text(app.preference_assessment.patype.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut app.preference_assessment.patype,
                        PaType::None,
                        "None",
                    );
                    ui.selectable_value(
                        &mut app.preference_assessment.patype,
                        PaType::PairedChoice,
                        "Paired Choice",
                    );
                });
            ui.return_button(app, |_| {});
        });
    }
}
