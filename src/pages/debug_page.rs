use crate::app::DataPro;

impl DataPro {
    pub fn view_debug_page(&mut self, ui: &mut egui::Ui) {
        let open = &mut self.display_info.debug_open;
        egui::Window::new("Debug Data").open(open).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("debug scroller")
                .show(ui, |ui| {
                    ui.add_space(10.0);

                    ui.monospace(format!("{:#?}", self.data));
                });
        });
    }
}
