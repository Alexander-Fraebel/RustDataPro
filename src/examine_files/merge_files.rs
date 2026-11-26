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
    pub merge_succeeded: bool,
}

impl MergeFiles {
    pub fn merge_files(&self) -> Result<SessionSummary> {
        SessionSummary::merge(&self.data.iter().map(|d| d.0.clone()).collect_vec())
    }

    pub fn prepare(&mut self, select_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("json files", vec!["json"])
            .default_file_filter("json files");
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.prepare(pathbuf);
        }

        if let Some(pathbufs) = self.select_file_dialog.take_picked_multiple() {
            // Clear everything
            self.data.clear();

            // Load the selected files if possible
            for buf in pathbufs {
                match SessionSummary::from_json_file(buf.as_path()) {
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
                "Merge multiple session summary files as if they were a single session. Produces the two normal session files (json and xlsx) that will use the same session number as the first file selected.",
            );
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Files").clicked() {
                    self.merger.select_file_dialog.pick_multiple();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Merge Files").clicked() {
                    match self.merger.merge_files() {
                        Ok(merged) => {
                            if let Some((_, path)) = self.merger.data.first() {
                            let mut path = path.clone();

                            path.pop();
                            path.push(merged.merged_json_file_name());
                            quick_error!(overwrite_file(
                                Ok(path.clone()),
                                &merged
                                    .to_json()
                                    .expect("unable to convert merged file to json"),
                            ));

                            path.pop();
                            path.push(merged.merged_xlsx_file_name());
                            quick_error!(
                                merged
                                    .to_xlsx()
                                    .expect("unable to convert merged file to xlsx")
                                    .save(path)
                                    .context("unable to save merged xlsx file")
                            );
                            }
                        }
                        Err(e) => windows_error_dialog(e),
                    }
                }
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.monospace("Files");
                    ui.group(|ui| {
                        egui::ScrollArea::both()
                            .id_salt("scroller")
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

            if self.merger.merge_succeeded {
                ui.monospace(
                    RichText::new(
                        "Merged files were saved to the same folder as the original files!",
                    )
                    .heading()
                    .color(egui::Color32::GREEN),
                );
            }

            ui.add_space(8.0);
        });
    }
}
