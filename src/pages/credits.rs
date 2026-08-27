use crate::app::DataPro;
use egui::RichText;

impl DataPro {
    pub fn view_credits(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.label(RichText::from("Welcome to RustDataPro").heading().strong());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Written in ");
                    ui.hyperlink_to("Rust", "https://rust-lang.org/");
                    ui.label(".");
                });
            });          
            if ui.button("Manually Set Dark Mode").clicked() {
                ui.ctx().set_visuals(egui::Visuals::dark());
            }
            ui.add_space(10.0);

            ui.label("Empowering you to accurately track client data with ease!");            
            ui.add_space(15.0);

            ui.label("Dark mode available for reduced eye strain in observation rooms.\nYou can control Assessment and KSF files directly in the application without needing to edit files.\nData output in ready to use Excel files (.xlsx) and a compact format accepted everywhere (.txt files following JSON formatting).\nUse the Shuffler for instant crytographic quality randomization.\nOn the Timers page you can name stopwatches and countdowns for improved usability and even synchronize timers that need to begin simultaneously by linking them.");
            ui.add_space(5.0);

            ui.label("COMING SOON: Preference Assessments (MSWO, Free Operant, Paired Choice)");
            ui.add_space(5.0);

            ui.label("IN THE FUTURE?: graphing");
            ui.add_space(10.0);
        });
    }
}
