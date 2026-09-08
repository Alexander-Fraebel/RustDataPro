use egui::Ui;

use crate::utils::Timer;

#[derive(Debug, Default)]
pub struct FreeOperant {
    pub operants: Vec<(String, Timer)>,
}

impl FreeOperant {}

impl crate::app::DataPro {
    pub fn view_free_operant(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("Free Operant");
        });
    }
}
