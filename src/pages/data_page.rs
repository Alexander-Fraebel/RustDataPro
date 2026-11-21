use crate::{app::DataPro, utils::ui_elements::DataProUiElements};

impl DataPro {
    pub fn view_data_page(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            if ui.large_button("Calculate IOA").clicked() {
                self.go_to_ioa();
            }

            if ui.large_button("Collate Files").clicked() {
                self.go_to_collate();
            }
        });
    }
}
