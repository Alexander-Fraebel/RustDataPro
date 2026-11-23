use crate::{
    app::DataPro,
    data::SessionSummary,
    quick_error,
    utils::{overwrite_file, ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::{Context, Result};
use egui::{RichText, Ui};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;

#[derive(Default)]
pub struct RebuilderPage {
    pub temp_file_path: Option<PathBuf>,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub rebuild_succeeded: bool,
}

impl RebuilderPage {
    pub fn rebuild_file(&self) -> Result<SessionSummary> {
        Ok(SessionSummary::rebuild(&std::fs::read_to_string(
            &self.select_path,
        )?)?)
    }

    pub fn prepare(&mut self, select_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("temp files", vec!["temp"])
            .default_file_filter("temp files");
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.prepare(pathbuf.clone());
            self.temp_file_path = Some(pathbuf.clone());
        }
    }
}

impl DataPro {
    pub fn view_file_rebuilder(&mut self, ui: &mut Ui) {
        self.rebuilder.file_dialog_controls(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Rebuild Files For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("Take a temp file and create the normal session summary files (json and xlsx) from it.");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Files").clicked() {
                    self.rebuilder.select_file_dialog.pick_file();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Rebuild Files").clicked() {
                    match self.rebuilder.rebuild_file() {
                        Ok(rebuilt) => {
                            self.rebuilder.rebuild_succeeded = false;
                            let mut path = self.rebuilder.select_path.clone();

                            path.pop();
                            path.push(rebuilt.json_file_name());
                            quick_error!(overwrite_file(
                                Ok(path.clone()),
                                &rebuilt
                                    .to_json()
                                    .expect("unable to convert rebuilt file to json"),
                            ));

                            path.pop();
                            path.push(rebuilt.xlsx_file_name());
                            quick_error!(
                                rebuilt
                                    .to_xlsx()
                                    .expect("unable to convert rebuilt file to xlsx")
                                    .save(path)
                                    .context("error saving rebuilt file as xlsx")
                            );
                            self.rebuilder.rebuild_succeeded = true;
                        }
                        Err(e) => {
                            windows_error_dialog(e);
                            self.rebuilder.rebuild_succeeded = false;
                        }
                    }
                }
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.monospace("File");
                    ui.group(|ui| {
                        egui::ScrollArea::both()
                            .id_salt("time series scroller")
                            .max_width(300.0)
                            .show(ui, |ui| match self.rebuilder.temp_file_path {
                                Some(ref file_name) => {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(
                                                file_name.file_name().unwrap().to_string_lossy(),
                                            )
                                            .monospace(),
                                        )
                                        .extend(),
                                    );
                                }
                                None => {
                                    ui.monospace("                              ");
                                }
                            });
                    });
                })
            });

            if self.rebuilder.rebuild_succeeded {
                ui.monospace(
                    RichText::new(
                        "Rebuilt files were saved to the same folder as the original file!",
                    )
                    .heading()
                    .color(egui::Color32::GREEN),
                );
            }

            ui.add_space(8.0);
        });
    }
}
