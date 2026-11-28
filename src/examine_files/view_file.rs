use crate::{
    app::DataPro,
    data::SessionSummary,
    utils::{ui_elements::DataProUiElements, windows_error_dialog},
};

use egui_file_dialog::FileDialog;
use std::path::PathBuf;

#[derive(Default)]
pub struct VisualizeFile {
    pub file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub summary: Option<SessionSummary>,
    pub changes_made: bool,
}

impl VisualizeFile {
    pub fn prepare(&mut self, select_path: PathBuf) {
        // Reset
        *self = Self::default();

        // Load the path information
        self.file_dialog = FileDialog::new().initial_directory(select_path.clone());
        self.select_path = select_path.clone();
        self.file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("json files", vec!["json"])
            .default_file_filter("json files");
    }
}

impl DataPro {
    pub fn view_visualizer_page(&mut self, ui: &mut egui::Ui) {
        self.visualizer.file_dialog.update(ui.ctx());
        if let Some(pathbuf) = self.visualizer.file_dialog.take_picked() {
            match SessionSummary::from_json_file(&pathbuf) {
                Ok(summary) => self.visualizer.summary = Some(summary),
                Err(e) => {
                    windows_error_dialog(e);
                    self.visualizer.summary = None
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("View a File For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("View the data from a JSON file. These files can also be opened in Notepad.");
            ui.add_space(5.0);

            if ui.large_button("Select File").clicked() {
                self.visualizer.file_dialog.pick_file();
            }

            egui::Grid::new("summary_info_grid")
                .min_col_width(120.0)
                .min_row_height(22.0)
                .show(ui, |ui| {
                    if let Some(summary) = &self.visualizer.summary {
                        ui.label("Client ID:");
                        ui.monospace(&summary.client_id);
                        ui.end_row();

                        ui.label("Session Start Time:");
                        ui.monospace(&summary.session_start_time);
                        ui.end_row();

                        ui.label("Notes:");
                        ui.monospace(&summary.notes);
                        ui.end_row();
                    } else {
                        ui.label("Client ID:");
                        ui.monospace("");
                        ui.end_row();

                        ui.label("Session Start Time:");
                        ui.monospace("");
                        ui.end_row();

                        ui.label("Notes:");
                        ui.monospace("");
                        ui.end_row();
                    }
                });

            ui.add_space(10.0);
        });
    }
}
