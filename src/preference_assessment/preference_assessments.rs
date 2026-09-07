use std::fmt::Display;

use egui::Ui;

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

pub struct PreferenceAssessments {}

impl crate::app::DataPro {
    pub fn view_preference_assessments_page(&mut self, ui: &mut Ui) {}
}
