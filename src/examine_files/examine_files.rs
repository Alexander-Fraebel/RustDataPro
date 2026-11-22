use crate::{app::DataPro, utils::ui_elements::DataProUiElements};

impl DataPro {
    pub fn view_file_exam_options(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(16.0);

            if ui.large_button("Interobserver Agreement").clicked() {
                self.go_to_ioa();
            }
            ui.add_space(16.0);

            if ui.large_button("Collate Files").clicked() {
                self.go_to_collate();
            }

            ui.add_space(16.0);

            if ui.large_button("Merge Files").clicked() {
                self.go_to_merger();
            }
        });
    }
}
