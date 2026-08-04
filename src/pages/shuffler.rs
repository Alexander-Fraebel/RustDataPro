use crate::app::DataPro;
use egui::{TextEdit, Ui};
use itertools::Itertools;
use rand::{make_rng, rngs::StdRng, seq::SliceRandom};

fn view_shuffler(page: &mut RandomServices, ui: &mut Ui) {
    ui.label("Separate items with commas.");
    ui.add_space(10.0);
    if ui.button("Shuffle").clicked() {
        let mut list: Vec<&str> = page.csv_list.split(',').collect();
        list.shuffle(&mut page.prng);
        let rep = list
            .iter()
            .map(|s| s.trim())
            .filter(|s| s.len() > 0)
            .join(", ");
        page.csv_list = rep;
    }
    ui.add_space(5.0);
    ui.add(
        TextEdit::multiline(&mut page.csv_list)
            .desired_width(300.0)
            .desired_rows(4),
    );
}

pub struct RandomServices {
    prng: StdRng, // ChaCha12 is more than enough for our purposes, initalized from SysRng
    csv_list: String,
}

impl Default for RandomServices {
    fn default() -> Self {
        Self {
            prng: make_rng(),
            csv_list: String::from("a, b, c, 1, 2, 3"),
        }
    }
}

impl RandomServices {
    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        egui::Window::new("Shuffler")
            .open(&mut app.display_info.random_open)
            .show(ui, |ui| {
                ui.add_space(10.0);
                view_shuffler(&mut app.randomness_page, ui)
            });
    }
}
