use std::fmt::Display;

use crate::app::DataPro;
use egui::Ui;

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
}

impl PreferenceAssessment {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        egui::ComboBox::from_id_salt("preference_assessment")
            .selected_text(app.preference_assessment.patype.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.preference_assessment.patype, PaType::None, "None");
                ui.selectable_value(
                    &mut app.preference_assessment.patype,
                    PaType::PairedChoice,
                    "Paired Choice",
                );
            });
    }
}
