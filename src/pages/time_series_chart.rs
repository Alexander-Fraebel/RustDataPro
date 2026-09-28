use crate::{
    app::DataPro,
    data::{Ksf, SessionResults},
    quick_error,
    utils::{ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::{Context, Result, anyhow};
use egui::{Color32, Key, RichText, Ui};
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use rust_xlsxwriter::{
    Format, FormatAlign,
    chart::{Chart, ChartMarker, ChartMarkerType},
    workbook::Workbook,
};
use std::{fmt::Display, path::PathBuf};

#[derive(Default, PartialEq, Eq)]
pub enum YAxis {
    Count,
    #[default]
    Rpm,
}

impl Display for YAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YAxis::Count => write!(f, "Frequency"),
            YAxis::Rpm => write!(f, "RPM"),
        }
    }
}

#[derive(Default)]
pub struct TimeSeriesChart {
    pub data: Vec<(SessionResults, PathBuf)>,
    pub ksf: Option<Ksf>,
    pub y_axis: YAxis,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
    pub keys_to_use: Vec<Key>,
    pub key_to_use_string: String,
    pub chart_created: bool,
}

impl TimeSeriesChart {
    pub fn prepare(&mut self, select_path: PathBuf, save_new_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new()
            .initial_directory(select_path.clone())
            .add_file_filter_extensions("json files", vec!["json"])
            .default_file_filter("json files");
        self.save_path = save_new_path.clone();
        self.save_file_dialog = FileDialog::new().initial_directory(save_new_path.clone());
    }

    pub fn key_picker(&mut self, ui: &mut Ui) {
        ui.heading("Choose Keys to Graph");
        ui.label("Separate keys with commas.");
        if ui
            .text_edit_multiline(&mut self.key_to_use_string)
            .changed()
        {
            self.keys_to_use.clear();
            self.chart_created = false;
            for name in self
                .key_to_use_string
                .split(",")
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                match Key::from_name(name) {
                    Some(key) => self.keys_to_use.push(key),
                    None => (), // TODO: do something
                }
            }
        };
    }

    pub fn collate_data(&self) -> Result<Workbook> {
        if self.data.is_empty() {
            return Err(anyhow!("no files selected"));
        }

        if self.ksf.is_none() {
            return Err(anyhow!("no KSF determined"));
        }

        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();
        let ksf_map = self.data[0].0.ksf.create_map();

        let mut workbook = Workbook::default();

        let data_page = workbook.add_worksheet();
        data_page.set_name("Data")?;
        data_page.set_column_range_width_pixels(0, 100, 80)?;
        data_page.set_freeze_panes(1, 1)?;
        for row in 1..self.data.len() + 1 {
            data_page.set_row_format(row as u32, &Format::new().set_num_format("0.00"))?;
        }

        // Always use these in order to maintain aligment as we go
        let mut col = 0;
        let mut row = 0;

        data_page.write_with_format(0, col, "Session", &centered_bold)?;
        col += 1;
        data_page.write_with_format(0, col, "Assessment", &centered_bold)?;
        col += 1;
        data_page.write_with_format(0, col, "Condition", &centered_bold)?;
        col += 1;

        let (freq, dura) = self.ksf.as_ref().unwrap().keys();
        for key in freq.chain(dura) {
            data_page.write_with_format(0, col, ksf_map.get(key).unwrap(), &centered_bold)?;
            col += 1;
        }

        for (result, buf) in self.data.iter() {
            row += 1;
            col = 0;
            data_page.write(row, col, result.session_number)?;
            col += 1;
            data_page.write(row, col, &result.session_data.chosen_assessment)?;
            col += 1;
            data_page.write(row, col, &result.session_data.chosen_condition)?;
            col += 1;

            let (freq, dura) = self.ksf.as_ref().unwrap().keys();
            for key in freq {
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
            for key in dura {
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

        Ok(workbook)
    }

    pub fn create_time_series_graph(&self) -> Result<Workbook> {
        let mut workbook = self.collate_data()?;

        if self.keys_to_use.is_empty() {
            return Err(anyhow!("no keys selected"));
        }

        let chart_page = workbook.add_chartsheet();
        chart_page.set_name("Graph")?;

        let chart_markers = [
            ChartMarkerType::Circle,
            ChartMarkerType::Square,
            ChartMarkerType::Triangle,
        ];

        let mut chart = Chart::new_line();
        chart
            .x_axis()
            .set_name("Session")
            .set_major_gridlines(false);
        chart
            .y_axis()
            .set_name(&self.y_axis.to_string())
            .set_major_gridlines(false);

        let max_row = self.data.len() as u32;

        for idx in 0..self.keys_to_use.len() {
            let col = (idx + 3) as u16;
            chart
                .add_series()
                .set_values(("Data", 1_u32, col, max_row, col))
                .set_categories(("Data", 1, 0, max_row, 0))
                .set_name(("Data", 0, col))
                .set_marker(
                    ChartMarker::new()
                        .set_type(chart_markers[idx % 3])
                        .set_size(5),
                );
        }

        chart_page.insert_chart(2, 2, &chart)?;
        chart_page.set_active(true);

        Ok(workbook)
    }

    pub fn file_dialog_controls(&mut self, ui: &mut Ui) {
        self.select_file_dialog.update(ui.ctx());
        self.save_file_dialog.update(ui.ctx());

        if let Some(pathbuf) = self.select_file_dialog.take_picked() {
            self.prepare(pathbuf, self.save_path.clone());
        }

        if let Some(pathbuf) = self.save_file_dialog.take_picked() {
            self.prepare(self.select_path.clone(), pathbuf);
        }

        if let Some(pathbufs) = self.select_file_dialog.take_picked_multiple() {
            self.ksf = None;
            self.data.clear();
            self.chart_created = false;
            for buf in pathbufs {
                match SessionResults::from_file_path(buf.as_path()) {
                    Ok(data) => self.data.push((data, buf)),
                    Err(e) => windows_error_dialog(e),
                }
            }
            self.ksf = Some(self.data[0].0.ksf.clone());
        }
    }
}

impl DataPro {
    pub fn view_time_series_page(&mut self, ui: &mut Ui) {
        self.time_series.file_dialog_controls(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            // TODO: more description
            ui.label("Create a simple line graph showing trends in the chosen Frequency keys across the selected sessions.");

            ui.heading("Create Time Series For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // ui.label("Select Files From:");
                    // ui.directory_picker(
                    //     &mut self.time_series.select_file_dialog,
                    //     &self.time_series.select_path,
                    // );
                    // ui.add_space(10.0);

                    // ui.label("Save Graph To:");
                    // ui.directory_picker(
                    //     &mut self.time_series.save_file_dialog,
                    //     &self.time_series.save_path,
                    // );
                    // ui.add_space(10.0);

                    if ui.large_button("Select Data").clicked() {
                        self.time_series.select_file_dialog.pick_multiple();
                    }
                    ui.add_space(10.0);

                    self.time_series.key_picker(ui);
                });

                ui.vertical(|ui| {
                    ui.monospace("KSF");
                    ui.group(|ui| {

                        ui.horizontal(|ui| {

                            ui.add_space(5.0);
                            if let Some(ksf) = &self.time_series.ksf {
                                let (freq, dura) = ksf.pairs();
                                ui.vertical(|ui| {
                                    ui.strong("Frequency Keys");
                                    ui.add_space(2.0);
                                    for (key, desc) in freq {
                                        ui.add(egui::Label::new(
                                            RichText::from(format!("{:>2} {}", key.symbol_or_name(), desc))
                                                .monospace()
                                                .size(12.0),
                                        ));
                                    }
                                });
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.strong("Duration Keys");
                                    ui.add_space(2.0);
                                    for (key, desc) in dura {
                                        ui.add(egui::Label::new(
                                            RichText::from(format!("{:>2} {}", key.symbol_or_name(), desc))
                                                .monospace()
                                                .size(12.0),
                                        ));
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
                        if self.time_series.data.is_empty() {
                            for _ in 0..4 {
                                ui.monospace("                              ");
                            }
                        } else {
                            egui::ScrollArea::vertical()
                                .id_salt("time series scroller").min_scrolled_height(128.0)
                                .show(ui, |ui| {
                                    for (_, file_name) in self.time_series.data.iter() {
                                        ui.monospace(file_name.file_name().unwrap().to_string_lossy());
                                    }
                                });
                        }

                    });
                })
            });

            ui.add_space(8.0);



            ui.heading("Y-Axis");
            egui::ComboBox::from_id_salt("time_series_type")
                    .selected_text(self.time_series.y_axis.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.time_series.y_axis, YAxis::Count, "Count");
                        ui.selectable_value(&mut self.time_series.y_axis, YAxis::Rpm, "Rate per Min");
                    }
                );
            ui.add_space(8.0);

            if ui.large_green_button("Create Time Series").clicked() {
                match self.time_series.create_time_series_graph() {
                    Ok(mut wkbk) => {
                        quick_error!(wkbk.save(self.time_series.save_path.join("time_series_graph.xlsx")).context("error while saving"))
                    }
                    Err(e) => windows_error_dialog(e),
                }
            }

            if ui.large_green_button("Collate Data from Files").clicked() {
                match self.time_series.collate_data() {
                    Ok(mut wkbk) => {
                        quick_error!(wkbk.save(self.time_series.save_path.join("collated.xlsx")).context("error while saving"))
                    }
                    Err(e) => windows_error_dialog(e),
                }
            }

            if self.time_series.chart_created {
                ui.monospace(RichText::new("Chart Created").color(Color32::GREEN));
            } else {
                ui.monospace(" ");
            }

        });
    }
}
