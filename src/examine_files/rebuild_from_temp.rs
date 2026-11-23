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
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl RebuilderPage {
    pub fn rebuild_file(&self) -> Result<SessionSummary> {
        Ok(SessionSummary::rebuild(&std::fs::read_to_string(
            &self.select_path,
        )?)?)
    }

    pub fn prepare(&mut self, select_path: PathBuf, save_new_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("temp files", vec!["temp"])
            .default_file_filter("temp files");
        self.save_path = save_new_path.clone();
        self.save_file_dialog = FileDialog::new()
            .initial_directory(save_new_path.clone())
            .default_file_name("rebuilt.json");
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());
        self.save_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.temp_file_path = Some(pathbuf.clone());
            self.prepare(pathbuf, self.save_path.clone());
        }

        // if let Some(mut pathbuf) = self.save_file_dialog.take_picked() {
        //     match self.rebuild_file() {
        //         Ok(rebuilt) => {
        //             quick_error!(overwrite_file(
        //                 Ok(rebuilt.json_file_name().into()),
        //                 &rebuilt
        //                     .to_json()
        //                     .expect("unable to convert rebuilt file to json"),
        //             ));
        //             quick_error!(
        //                 rebuilt
        //                     .to_xlsx()
        //                     .expect("unable to convert rebuilt file to xlsx")
        //                     .save(rebuilt.xlsx_file_name().into())
        //                     .context("")
        //             );
        //         }
        //         Err(e) => windows_error_dialog(e),
        //     }
        // }
    }
}

impl DataPro {
    pub fn view_file_rebuilder(&mut self, ui: &mut Ui) {
        self.rebuilder.file_dialog_controls(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Rebuild Files For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("Take a temp file and create a normal session summary from it.");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Files").clicked() {
                    self.rebuilder.select_file_dialog.pick_file();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Rebuild Files").clicked() {
                    match self.rebuilder.rebuild_file() {
                        Ok(rebuilt) => {
                            quick_error!(overwrite_file(
                                Ok(rebuilt.json_file_name().into()),
                                &rebuilt
                                    .to_json()
                                    .expect("unable to convert rebuilt file to json"),
                            ));
                            quick_error!(
                                rebuilt
                                    .to_xlsx()
                                    .expect("unable to convert rebuilt file to xlsx")
                                    .save(rebuilt.xlsx_file_name())
                                    .context("error saving rebuilt file as xlsx")
                            );
                        }
                        Err(e) => windows_error_dialog(e),
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

            ui.add_space(8.0);
        });
    }
}
