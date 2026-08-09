use crate::app::DataPro;
use egui::warn_if_debug_build;

pub struct DebugPage {}

impl DebugPage {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        let open = &mut app.display_info.debug_open;
        egui::Window::new("Debug Data").open(open).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("debug scroller")
                .show(ui, |ui| {
                    warn_if_debug_build(ui);
                    ui.add_space(10.0);

                    ui.monospace(format!("{:#?}", app.data));
                });
        });
    }
}
