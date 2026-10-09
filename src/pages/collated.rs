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
use rust_xlsxwriter::{
    Color, Format, FormatAlign, Formula, chart::Chart, workbook::Workbook, worksheet::Worksheet,
};
use std::path::PathBuf;

// These are colors used in Excel at time of writing
const GREEN_ACCENT: Color = Color::RGB(0xD8E4BC);
const BLUE_ACCENT: Color = Color::RGB(0xB8CCE4);
const ORANGE_ACCENT: Color = Color::RGB(0xFCD5B4);
const PURPLE_ACCENT: Color = Color::RGB(0xCCC0DA);

const LAST_ROW: u32 = u32::MAX;

fn to_xlsx_col(mut col: u16) -> String {
    let mut s = String::with_capacity(3);
    if col == 0 {
        s.push('A');
    }
    while col > 0 {
        s.push((97_u8 + (col as u8 % 26)) as char);
        col -= 1;
        col /= 26;
    }
    s
}

fn info_column(
    worksheet: &mut Worksheet,
    row: u32,
    col: &mut u16,
    name: &str,
    format: &Format,
) -> Result<()> {
    worksheet.write_with_format(row, *col, name, format)?;
    worksheet.set_column_format(*col, &Format::new().set_align(FormatAlign::Center))?;
    worksheet.set_column_width(*col, 80)?;
    *col += 1;
    Ok(())
}

fn data_column(
    worksheet: &mut Worksheet,
    row: u32,
    col: u16,
    key_desc: &str,
    key_symbol: &str,
    name_format: &Format,
    symbol_format: &Format,
    width: u32,
    num_format: &'static str,
) -> Result<()> {
    worksheet.write_with_format(row - 1, col, key_symbol, &symbol_format)?;
    worksheet.write_with_format(row, col, key_desc, &name_format)?;
    worksheet.set_column_format(col, &Format::new().set_num_format(num_format))?;
    worksheet.set_column_width_pixels(col, width)?;
    Ok(())
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

        let key_descriptions = self.data[0].0.ksf.create_map();
        let mut key_columns: IndexMap<&'static str, (u16, String)> = IndexMap::new();

        // Cell formatting to use
        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();
        let color_f = centered_bold.clone().set_background_color(GREEN_ACCENT);
        let color_d = centered_bold.clone().set_background_color(BLUE_ACCENT);
        let color_r = centered_bold.clone().set_background_color(ORANGE_ACCENT);
        let color_p = centered_bold.clone().set_background_color(PURPLE_ACCENT);
        let name_format = centered_bold
            .clone()
            .set_align(FormatAlign::VerticalCenter)
            .set_rotation(90);
        let name_format_f = name_format.clone().set_background_color(GREEN_ACCENT);
        let name_format_d = name_format.clone().set_background_color(BLUE_ACCENT);
        let name_format_r = name_format.clone().set_background_color(ORANGE_ACCENT);
        let name_format_p = name_format.clone().set_background_color(PURPLE_ACCENT);

        // Create workbook and worksheet
        let mut workbook = Workbook::default();
        let data_page = workbook.add_worksheet();
        data_page.set_name("Data")?;

        // Use these in order to maintain aligment as we go
        let mut col = 0;
        let mut row = 2;

        // Create the general information columns. Freeze the DOA and Session number panes along with the top rows.
        info_column(data_page, row, &mut col, "DOA", &centered_bold)?;
        info_column(data_page, row, &mut col, "Session", &centered_bold)?;
        data_page.set_freeze_panes(row + 1, col)?;
        info_column(data_page, row, &mut col, "Assessment", &centered_bold)?;
        info_column(data_page, row, &mut col, "Condition", &centered_bold)?;
        data_page.set_column_range_width_pixels(0, col, 80)?;

        // Create the headings for the freq and dura keys
        let (freq, dura) = self.ksf.as_ref().unwrap().keys();
        let start_freq = col;
        for key in freq {
            data_column(
                data_page,
                row,
                col,
                key_descriptions.get(key).unwrap(),
                key.symbol_or_name(),
                &name_format_f,
                &color_f,
                30,
                "0",
            )?;
            key_columns.insert(key.symbol_or_name(), (col, to_xlsx_col(col)));
            col += 1;
        }
        data_page.merge_range(0, start_freq, 0, col - 1, "Frequency", &color_f)?;
        let start_dura = col;
        for key in dura {
            data_column(
                data_page,
                row,
                col,
                key_descriptions.get(key).unwrap(),
                key.symbol_or_name(),
                &name_format_d,
                &color_d,
                50,
                "0.0",
            )?;
            key_columns.insert(key.symbol_or_name(), (col, to_xlsx_col(col)));
            col += 1;
        }
        // Include Active Time headings
        data_column(
            data_page,
            row,
            col,
            "(Seconds)",
            "AT",
            &name_format_d,
            &color_d,
            50,
            "0.0",
        )?;
        key_columns.insert("(Seconds)", (col, to_xlsx_col(col)));
        col += 1;
        data_column(
            data_page,
            row,
            col,
            "(Minutes)",
            "AT ",
            &name_format_d,
            &color_d,
            50,
            "0.0",
        )?;
        key_columns.insert("(Minutes)", (col, to_xlsx_col(col)));
        col += 1;
        data_page.merge_range(0, start_dura, 0, col - 1, "Duration", &color_d)?;

        // Headings for the rate information
        let start_rate = col;
        let (freq, dura) = self.ksf.as_ref().unwrap().keys();
        for key in freq {
            data_column(
                data_page,
                row,
                col,
                key_descriptions.get(key).unwrap(),
                "",
                &name_format_r,
                &color_r,
                50,
                "0.0",
            )?;
            col += 1;
        }
        data_page.merge_range(0, start_rate, 0, col - 1, "Rate (Per Minute)", &color_r)?;
        let start_pct_at = col;
        // Heading for Percent data
        for key in dura {
            data_column(
                data_page,
                row,
                col,
                key_descriptions.get(key).unwrap(),
                "",
                &name_format_p,
                &color_p,
                50,
                "0.0%",
            )?;
            col += 1;
        }
        data_page.merge_range(0, start_pct_at, 0, col - 1, "Percent of Session", &color_p)?;

        // Populate the basic data
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
                if key_columns.contains_key(key.symbol_or_name()) {
                    if !result.ksf.freq.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the frequency key {} is not in the KSF for file {}",
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
                if key_columns.contains_key(key.symbol_or_name()) {
                    if !result.ksf.dura.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the duration key {} is not in the KSF for file {}",
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
            // Active time in seconds
            let at = result.active_time;
            data_page.write(row, col, at)?;
            col += 1;
            // Formula for AT in minutes. This makes extending manually much easier.
            data_page.write(
                row,
                col,
                Formula::new(format!(
                    "={}{}/60",
                    &key_columns
                        .get("(Seconds)")
                        .expect("no (Seconds) key in key_columns")
                        .1,
                    row + 1
                )), // have to add one because excel numbers start at 1
            )?;
            col += 1;

            // Populate the rate information
            let (freq, dura) = self.ksf.as_ref().unwrap().keys();
            for key in freq {
                if key_columns.contains_key(key.symbol_or_name()) {
                    if !result.ksf.freq.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the frequency key {} is not in the KSF for file {}",
                            key.symbol_or_name(),
                            buf.as_os_str().to_string_lossy()
                        ));
                    } else {
                        let rpm = Formula::new(format!(
                            "={}{}/{}{}",
                            key_columns.get(key.symbol_or_name()).unwrap().1,
                            row + 1, // have to add one because excel numbers start at 1
                            key_columns
                                .get("(Minutes)")
                                .expect("no (Minutes) key in key_columns")
                                .1,
                            row + 1
                        ));
                        data_page.write(row, col, rpm)?;
                        col += 1;
                    }
                }
            }
            for key in dura {
                if key_columns.contains_key(key.symbol_or_name()) {
                    if !result.ksf.dura.iter().map(|(k, _)| k).contains(key) {
                        return Err(anyhow!(
                            "the duration key {} is not in the KSF for file {}",
                            key.symbol_or_name(),
                            buf.as_os_str().to_string_lossy()
                        ));
                    } else {
                        let ratio = Formula::new(format!(
                            "={}{}/{}{}",
                            key_columns.get(key.symbol_or_name()).unwrap().1,
                            row + 1, // have to add one because excel numbers start at 1
                            key_columns
                                .get("(Seconds)")
                                .expect("no (Seconds) key in key_columns")
                                .1,
                            row + 1
                        ));
                        data_page.write(row, col, ratio)?;
                        col += 1;
                    }
                }
            }
            row += 1;
        }

        if self.keys_selector.iter().filter(|(_, b)| **b).count() != 0 {
            let chart = workbook.add_chartsheet();
            let mut lines = Chart::new_line();

            for selected_key in self
                .keys_selector
                .iter()
                .filter(|(_, b)| **b)
                .map(|(k, _)| k.symbol_or_name())
            {
                if let Some((_, col)) = key_columns.get(selected_key) {
                    lines
                        .add_series()
                        // .set_categories(&format!("Data!$B$4:$B${}", LAST_ROW))
                        .set_values(&format!("Data!${}$4:${}${}", col, col, LAST_ROW));
                }
            }
            chart.insert_chart(1, 1, &lines)?;
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
                    quick_error!(
                        wkbk.save(pathbuf)
                            .context("error saving collated data file")
                    )
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
                self.keys_selector.insert(*key, false);
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

            ui.label(
                "Gather the data from multiple files into a single nicely formated Excel document. Shows frequency and duration data. Calculates rate per minute for frequncy and percentage of session of duration.",
            );
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
            ui.add_space(5.0);

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
