use std::fmt::Display;

use egui::Ui;

use crate::preference_assessment::{PairedChoice, free_operant::FreeOperant};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum PaType {
    #[default]
    None,
    PairedChoice,
    FreeOperant,
}

impl Display for PaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaType::None => write!(f, "None"),
            PaType::PairedChoice => write!(f, "Paired Choice"),
            PaType::FreeOperant => write!(f, "Free Operant"),
        }
    }
}

#[derive(Debug, Default)]
pub struct PreferenceAssessments {
    pub pa_type: PaType,
    pub free_operant: FreeOperant,
    pub paired_choice: PairedChoice,
}

impl PreferenceAssessments {
    pub fn pa_selector(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_id_salt("condition")
            .selected_text(self.pa_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.pa_type, PaType::None, "None");
                ui.selectable_value(&mut self.pa_type, PaType::FreeOperant, "Free Operant");
                ui.selectable_value(&mut self.pa_type, PaType::PairedChoice, "Paired Choice");
            });
    }
}

impl crate::app::DataPro {
    pub fn view_preference_assessments_page(&mut self, ui: &mut Ui) {
        ui.label("Preference Assessments");
        self.preference_assessment.pa_selector(ui);

        match self.preference_assessment.pa_type {
            PaType::None => {
                ui.label("No PA Type Selected");
            }
            PaType::PairedChoice => self.view_paired_choice(ui),
            PaType::FreeOperant => self.view_free_operant(ui),
        }
    }
}
