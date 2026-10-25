use crate::app::DataPro;
use egui::{TextEdit, Ui};
use itertools::Itertools;
use rand::seq::SliceRandom;

pub struct Shuffler {
    list: String,
}

impl Default for Shuffler {
    fn default() -> Self {
        Self {
            list: String::new(),
        }
    }
}

impl DataPro {
    pub fn view_shuffler(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.label(
                "Type each item on a seperate line. Empty lines will be removed when shuffling.",
            );
            ui.add_space(10.0);
            if ui.button("Shuffle").clicked() {
                let mut list: Vec<&str> = self.randomness_page.list.split('\n').collect();
                list.shuffle(&mut self.rng);
                let rep = list
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| s.len() > 0)
                    .join("\n");
                self.randomness_page.list = rep;
            }
            ui.add_space(5.0);
            ui.add(
                TextEdit::multiline(&mut self.randomness_page.list)
                    .desired_width(400.0)
                    .desired_rows(6),
            );
        });
    }
}
