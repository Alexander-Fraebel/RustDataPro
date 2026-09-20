use crate::{
    app::DataPro,
    data::SessionResults,
    quick_error,
    utils::{ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::Result;
use egui::{Color32, RichText, Ui};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;

pub struct VisualizeTimeline {
    pub graph_created: bool,
    pub output_data: Option<SessionResults>,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
}

impl Default for VisualizeTimeline {
    fn default() -> Self {
        Self {
            graph_created: false,
            output_data: None,
            select_file_dialog: FileDialog::new(),
            select_path: PathBuf::default(),
        }
    }
}

impl VisualizeTimeline {
    pub fn prepare(&mut self, select_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new().initial_directory(select_path.clone());
    }

    pub fn load_file(&mut self, pathbuf: PathBuf) {
        match SessionResults::from_file_path(pathbuf.as_path()) {
            Ok(data) => self.output_data = Some(data),
            Err(e) => windows_error_dialog(e),
        }
    }

    pub fn save_output(&self, save_path: PathBuf) -> Result<()> {
        if let Some(data) = &self.output_data {
            match data.timeline_to_xlsx() {
                Ok(mut workbook) => {
                    workbook.save(save_path)?;
                }
                Err(e) => {
                    windows_error_dialog(e);
                }
            }
        }
        Ok(())
    }
}

impl DataPro {
    pub fn view_timeline_visualizer(&mut self, ui: &mut Ui) {
        self.visualize_timeline.select_file_dialog.update(ui.ctx());
        if let Some(pathbuf) = self.visualize_timeline.select_file_dialog.take_picked() {
            self.visualize_timeline.load_file(pathbuf);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Visualize Session Timeline For");
            self.client_picker(ui);
            ui.add_space(15.0);

            if ui.button("Load Data").clicked() {
                self.visualize_timeline.select_file_dialog.pick_file();
            }

            if let Some(data) = &self.visualize_timeline.output_data {
                ui.monospace(format!("Session: {}", data.session_number));
                ui.monospace(format!(
                    "Assessment: {}",
                    data.session_data.chosen_assessment
                ));
                ui.monospace(format!("Condition: {}", data.session_data.chosen_condition));
                ui.monospace(format!("KSF Name: {}", data.session_data.chosen_ksf_name));
            } else {
                ui.monospace("No Data Selected");
                ui.monospace(" ");
                ui.monospace(" ");
                ui.monospace(" ");
            }

            ui.add_space(20.0);

            let data_loaded = self.visualize_timeline.output_data.is_some();
            ui.add_enabled_ui(data_loaded, |ui| {
                if ui.large_green_button("Create Timeline Graph").clicked() {
                    if let Some(data) = &self.visualize_timeline.output_data {
                        let path = self
                            .path_to_session_records_dir()
                            .join(data.xlsx_timeline_name());
                        quick_error!(self.visualize_timeline.save_output(path));
                    }
                }
            });
            ui.add_space(5.0);

            if self.visualize_timeline.graph_created {
                ui.monospace(
                    RichText::new("Timeline Graphed!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    }
}
