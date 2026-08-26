use crate::app::DataPro;

impl DataPro {
    pub fn view_debug_page(&mut self, ui: &mut egui::Ui) {
        let open = &mut self.display_info.debug_open;
        egui::Window::new("Debug Data").open(open).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("debug scroller")
                .max_height(200.0)
                .min_scrolled_height(200.0)
                .show(ui, |ui| {
                    ui.monospace(format!("{:#?}", self.data));
                });
            ui.add_space(5.0);

            ui.label("Proportional Font");
            ui.label("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
            ui.add_space(5.0);

            ui.monospace("Monospace Font");
            ui.monospace("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
        });
    }
}
