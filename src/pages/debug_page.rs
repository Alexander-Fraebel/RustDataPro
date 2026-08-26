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

            ui.label("Quick Font Test");
            ui.label("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
            ui.monospace("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789");
        });
    }
}
