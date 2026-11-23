use crate::{
    app::DataPro,
    data::SessionSummary,
    quick_error,
    utils::{overwrite_file, ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::{Context, Result};
use egui::{RichText, Ui};
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use std::path::PathBuf;

#[derive(Default)]
pub struct MergeFiles {
    pub data: Vec<(SessionSummary, PathBuf)>,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl MergeFiles {
    pub fn merge_files(&self) -> Result<SessionSummary> {
        SessionSummary::merge(&self.data.iter().map(|d| d.0.clone()).collect_vec())
    }

    pub fn prepare(&mut self, select_path: PathBuf, save_new_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("json files", vec!["json"])
            .default_file_filter("json files");
        self.save_path = save_new_path.clone();
        self.save_file_dialog = FileDialog::new()
            .initial_directory(save_new_path.clone())
            .default_file_name("merged.json");
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());
        self.save_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.prepare(pathbuf, self.save_path.clone());
        }

        if let Some(mut pathbuf) = self.save_file_dialog.take_picked() {
            match self.merge_files() {
                Ok(merged) => {
                    quick_error!(overwrite_file(
                        Ok(pathbuf.clone()),
                        &merged
                            .to_json()
                            .expect("unable to convert merged file to json"),
                    ));
                    pathbuf.pop();
                    pathbuf.add_extension(".xlsx");
                    quick_error!(
                        merged
                            .to_xlsx()
                            .expect("unable to convert merged file to xlsx")
                            .save(pathbuf)
                            .context("")
                    );
                }
                Err(e) => windows_error_dialog(e),
            }
        }

        if let Some(pathbufs) = self.select_file_dialog.take_picked_multiple() {
            // Clear everything
            self.data.clear();

            // Load the selected files if possible
            for buf in pathbufs {
                match SessionSummary::from_file_path(buf.as_path()) {
                    Ok(data) => self.data.push((data, buf)),
                    Err(e) => windows_error_dialog(e),
                }
            }
        }
    }
}

impl DataPro {
    pub fn view_file_merger(&mut self, ui: &mut Ui) {
        self.merger.file_dialog_controls(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Merge Files For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label(
                "Merge multiple session summary files into a single file as if they represented a single session.",
            );
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Files").clicked() {
                    self.merger.select_file_dialog.pick_multiple();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Merge Files").clicked() {
                    self.merger.save_file_dialog.save_file();
                }
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.monospace("Files");
                    ui.group(|ui| {
                        egui::ScrollArea::both()
                            .id_salt("time series scroller")
                            .max_width(300.0)
                            .show(ui, |ui| {
                                if self.merger.data.is_empty() {
                                    for _ in 0..4 {
                                        ui.monospace("                              ");
                                    }
                                } else {
                                    for (_, file_name) in self.merger.data.iter() {
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(
                                                    file_name
                                                        .file_name()
                                                        .unwrap()
                                                        .to_string_lossy(),
                                                )
                                                .monospace(),
                                            )
                                            .extend(),
                                        );
                                    }
                                }
                            });
                    });
                })
            });

            ui.add_space(8.0);
        });
    }
}
