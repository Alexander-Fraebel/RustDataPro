use crate::{app::DataPro, ui_elements::DataProUiElements};
use egui::Ui;
use itertools::Itertools;
use rand::{rngs::StdRng, seq::SliceRandom};
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
    pub conditions_string: String,
    pub conditions: Vec<String>,
    pub all_pairs: Vec<(String, String)>,
}

impl PreferenceAssessment {
    pub fn update_conditions(&mut self) {
        self.conditions = self
            .conditions_string
            .split(",")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn update_pairs(&mut self, rng: &mut StdRng) {
        self.all_pairs.clear();
        for a in self.conditions.iter() {
            for b in self.conditions.iter() {
                if a != b {
                    self.all_pairs.push((a.clone(), b.clone()));
                }
            }
        }
        self.all_pairs.shuffle(rng);
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {

            ui.return_button(app, |_| {});
            ui.add_space(5.0);

            ui.heading("Paired Choice");
            ui.label("Separate conditions with commas. All ordered pairs of conditions will be produced automatically.");
            if ui
                .text_edit_multiline(&mut app.preference_assessment.conditions_string)
                .changed()
            {
                app.preference_assessment.update_conditions();
                app.preference_assessment.update_pairs(&mut app.rng);
            }
            ui.add_space(5.0);

            ui.label(format!(
                "{}",
                app.preference_assessment
                    .all_pairs
                    .iter()
                    .map(|(a, b)| format!("[{a}, {b}]"))
                    .join("\n")
            ))
        });
    }
}
