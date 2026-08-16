use crate::app::DataPro;
use egui::{TextEdit, Ui};
use itertools::Itertools;
use rand::seq::SliceRandom;

pub struct Shuffler {
    csv_list: String,
}

impl Default for Shuffler {
    fn default() -> Self {
        Self {
            csv_list: String::new(),
        }
    }
}

impl Shuffler {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        egui::Window::new("Shuffler")
            .open(&mut app.display_info.random_open)
            .show(ui, |ui| {
                ui.add_space(10.0);
                ui.label("Separate items with commas.");
                ui.add_space(10.0);
                if ui.button("Shuffle").clicked() {
                    let mut list: Vec<&str> = app.randomness_page.csv_list.split(',').collect();
                    list.shuffle(&mut app.rng);
                    let rep = list
                        .iter()
                        .map(|s| s.trim())
                        .filter(|s| s.len() > 0)
                        .join(", ");
                    app.randomness_page.csv_list = rep;
                }
                ui.add_space(5.0);
                ui.add(
                    TextEdit::multiline(&mut app.randomness_page.csv_list)
                        .hint_text("a, b, c, 1, 2, 3")
                        .desired_width(300.0)
                        .desired_rows(4),
                );
            });
    }
}
