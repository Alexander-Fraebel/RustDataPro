use crate::{
    app::DataPro,
    data::{Ksf, SessionResults},
    quick_error,
    utils::{ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::{Context, Result, anyhow};
use egui::{Key, RichText, Ui};
use egui_file_dialog::FileDialog;
use indexmap::IndexMap;
use itertools::Itertools;
use rust_xlsxwriter::{Color, Format, FormatAlign, Formula, workbook::Workbook};
use std::path::PathBuf;

pub fn idx_to_xlsx_col(mut idx: u16) -> String {
    let mut col = String::with_capacity(3);
    if idx == 0 {
        col.push('A');
    }
    while idx > 0 {
        col.push((97_u8 + (idx as u8 % 26)) as char);
        idx -= 1;
        idx /= 26;
    }
    col
}

#[derive(Default)]
pub struct CollatePage {
    pub data: Vec<(SessionResults, PathBuf)>,
    pub ksf: Option<Ksf>,
    pub keys_selector: IndexMap<Key, bool>,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl CollatePage {
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
            .default_file_name("collated.xlsx");
    }

    pub fn collate_data(&self) -> Result<Workbook> {
        if self.data.is_empty() {
            return Err(anyhow!("no files selected"));
        }

        if self.ksf.is_none() {
            return Err(anyhow!("no KSF determined"));
        }

        let green_accent = Color::RGB(0xD8E4BC);
        let blue_accent = Color::RGB(0xB8CCE4);

        let key_descriptions = self.data[0].0.ksf.create_map();
        let mut key_info: IndexMap<Key, (u16, String)> = IndexMap::new();

        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();
        let freq_cell_name_foramt = centered_bold
            .clone()
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color(green_accent)
            .set_rotation(90);
        let freq_cell_key_foramt = centered_bold.clone().set_background_color(green_accent);
        let dura_cell_name_foramt = centered_bold
            .clone()
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color(blue_accent)
            .set_rotation(90);
        let dura_cell_key_foramt = centered_bold.clone().set_background_color(blue_accent);

        let mut workbook = Workbook::default();

        let data_page = workbook.add_worksheet();
        data_page.set_name("Data")?;

        // Use these in order to maintain aligment as we go
        let mut col = 0;
        let mut row = 1;

        data_page.write_with_format(row, col, "DOA", &centered_bold)?;
        data_page.set_column_format(col, &Format::new().set_align(FormatAlign::Center))?;
        col += 1;
        data_page.write_with_format(row, col, "Session", &centered_bold)?;
        data_page.set_column_format(col, &Format::new().set_align(FormatAlign::Center))?;
        col += 1;
        data_page.write_with_format(row, col, "Assessment", &centered_bold)?;
        data_page.set_column_format(col, &Format::new().set_align(FormatAlign::Center))?;
        col += 1;
        data_page.write_with_format(row, col, "Condition", &centered_bold)?;
        data_page.set_column_format(col, &Format::new().set_align(FormatAlign::Center))?;

        data_page.set_freeze_panes(row + 1, col + 1)?;
        data_page.set_column_range_width_pixels(0, col + 1, 80)?;

        // Create the headings for the freq and dura keys
        col += 1;
        let (freq, dura) = self.ksf.as_ref().unwrap().keys();
        for key in freq {
            if let Some(b) = self.keys_selector.get(key) {
                if *b {
                    data_page.write_with_format(
                        row - 1,
                        col,
                        key_descriptions.get(key).unwrap(),
                        &freq_cell_name_foramt,
                    )?;
                    data_page.write_with_format(
                        row,
                        col,
                        key.symbol_or_name(),
                        &freq_cell_key_foramt,
                    )?;
                    data_page.set_column_format(col, &Format::new().set_num_format("0"))?;
                    data_page.set_column_width_pixels(col, 35)?;
                    key_info.insert(key.clone(), (col, idx_to_xlsx_col(col)));
                    col += 1;
                }
            }
        }
        for key in dura {
            if let Some(b) = self.keys_selector.get(key) {
                if *b {
                    data_page.write_with_format(
                        row - 1,
                        col,
                        key_descriptions.get(key).unwrap(),
                        &dura_cell_name_foramt,
                    )?;
                    data_page.write_with_format(
                        row,
                        col,
                        key.symbol_or_name(),
                        &dura_cell_key_foramt,
                    )?;

                    data_page.set_column_format(col, &Format::new().set_num_format("0.0"))?;
                    data_page.set_column_width_pixels(col, 35)?;
                    key_info.insert(key.clone(), (col, idx_to_xlsx_col(col)));
                    col += 1;
                }
            }
        }
        // Include Active Time
        data_page.write_with_format(row, col, "", &dura_cell_key_foramt)?;
        data_page.write_with_format(row - 1, col, "AT (Secs)", &dura_cell_name_foramt)?;
        data_page.set_column_format(col, &Format::new().set_num_format("0.0"))?;
        data_page.set_column_width_pixels(col, 50)?;
        col += 1;
        data_page.write_with_format(row, col, "", &dura_cell_key_foramt)?;
        data_page.write_with_format(row - 1, col, "AT (Mins)", &dura_cell_name_foramt)?;
        data_page.set_column_format(col, &Format::new().set_num_format("0.0"))?;
        data_page.set_column_width_pixels(col, 50)?;

        // Populate the data
        row += 1;
        for (result, buf) in self.data.iter() {
            col = 0;
            data_page.write(row, col, result.days_since_admission as f32)?;
            col += 1;
            data_page.write(row, col, result.session_number)?;
            col += 1;
            data_page.write(row, col, &result.session_data.chosen_assessment)?;
            col += 1;
            data_page.write(row, col, &result.session_data.chosen_condition)?;
            col += 1;

            let (freq, dura) = self.ksf.as_ref().unwrap().keys();
            for key in freq {
                if key_info.contains_key(key) {
                    if !result.ksf.freq.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the key {} is not in the KSF for file {}",
                            key.symbol_or_name(),
                            buf.as_os_str().to_string_lossy()
                        ));
                    } else {
                        let count = *result.frequency_data.get(key).unwrap() as f32;
                        data_page.write(row, col, count)?;
                        col += 1;
                    }
                }
            }
            for key in dura {
                if key_info.contains_key(key) {
                    if !result.ksf.dura.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the key {} is not in the KSF for file {}",
                            key.symbol_or_name(),
                            buf.as_os_str().to_string_lossy()
                        ));
                    } else {
                        let time = result.duration_data.get(key).unwrap().1;
                        data_page.write(row, col, time)?;
                        col += 1;
                    }
                }
            }
            // Include Active Time
            let at = result.active_time;
            data_page.write(row, col, at)?;
            col += 1;
            data_page.write(
                row,
                col,
                Formula::new(format!("={}{}/60", idx_to_xlsx_col(col - 1), row + 1)),
            )?;
            row += 1;
        }

        Ok(workbook)
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());
        self.save_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.prepare(pathbuf, self.save_path.clone());
        }

        if let Some(pathbuf) = self.save_file_dialog.take_picked() {
            match self.collate_data() {
                Ok(mut wkbk) => {
                    quick_error!(wkbk.save(pathbuf).context("erro saving collated data file"))
                }
                Err(e) => windows_error_dialog(e),
            }
        }

        if let Some(pathbufs) = self.select_file_dialog.take_picked_multiple() {
            self.ksf = None;
            self.data.clear();
            for buf in pathbufs {
                match SessionResults::from_file_path(buf.as_path()) {
                    Ok(data) => self.data.push((data, buf)),
                    Err(e) => windows_error_dialog(e),
                }
            }
            self.ksf = Some(self.data[0].0.ksf.clone());
            self.keys_selector.clear();
            for key in self.data[0].0.ksf.all_keys() {
                self.keys_selector.insert(*key, true);
            }
        }
    }
}

impl DataPro {
    pub fn view_time_series_page(&mut self, ui: &mut Ui) {
        self.collate.file_dialog_controls(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Collate Files For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("Gather the data from multiple files into a single Excel document.");
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Files").clicked() {
                    self.collate.select_file_dialog.pick_multiple();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Collate Files").clicked() {
                    self.collate.save_file_dialog.save_file();
                }
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.monospace("KSF");
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(5.0);
                            if let Some(ksf) = &self.collate.ksf {
                                let (freq, dura) = ksf.pairs();
                                ui.vertical(|ui| {
                                    ui.strong("Frequency Keys");
                                    ui.add_space(2.0);
                                    for (key, desc) in freq {
                                        if let Some(b) = self.collate.keys_selector.get_mut(key) {
                                            ui.checkbox(
                                                b,
                                                RichText::from(format!(
                                                    "{:>2} {}",
                                                    key.symbol_or_name(),
                                                    desc
                                                ))
                                                .monospace(),
                                            );
                                        }
                                    }
                                });
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.strong("Duration Keys");
                                    ui.add_space(2.0);
                                    for (key, desc) in dura {
                                        if let Some(b) = self.collate.keys_selector.get_mut(key) {
                                            ui.checkbox(
                                                b,
                                                RichText::from(format!(
                                                    "{:>2} {}",
                                                    key.symbol_or_name(),
                                                    desc
                                                ))
                                                .monospace(),
                                            );
                                        }
                                    }
                                });
                            } else {
                                ui.vertical(|ui| {
                                    ui.strong("Frequency Keys");
                                    ui.add_space(50.0);
                                });
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.strong("Duration Keys");
                                    ui.add_space(50.0);
                                });
                            }
                            ui.add_space(5.0);
                        });
                    });
                });

                ui.vertical(|ui| {
                    ui.monospace("Files");
                    ui.group(|ui| {
                        egui::ScrollArea::both()
                            .id_salt("time series scroller")
                            .max_width(225.0)
                            .show(ui, |ui| {
                                if self.collate.data.is_empty() {
                                    for _ in 0..4 {
                                        ui.monospace("                              ");
                                    }
                                } else {
                                    for (_, file_name) in self.collate.data.iter() {
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
