use crate::{
    app::DataPro,
    data::SessionResults,
    quick_error,
    utils::{ui_elements::DataProUiElements, windows_error_dialog},
};
use anyhow::{Result, anyhow};
use egui::{Key, Ui};
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use rust_xlsxwriter::{
    Format, FormatAlign,
    chart::{Chart, ChartMarker, ChartMarkerType},
    workbook::Workbook,
};
use std::path::PathBuf;

#[derive(Default)]
pub struct TimeSeries {
    pub data: Vec<(SessionResults, PathBuf)>,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
    pub keys_to_use: Vec<Key>,
    pub key_to_use_string: String,
    pub chart_created: bool,
}

impl TimeSeries {
    pub fn prepare(&mut self, select_path: PathBuf, save_new_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new().initial_directory(select_path.clone());
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

    pub fn create_time_series_graph(&self) -> Result<()> {
        let centered_bold = Format::new().set_align(FormatAlign::Center).set_bold();

        let mut workbook = Workbook::default();
        let data_page = workbook.add_worksheet();
        data_page.set_name("Data")?;

        for (idx, key) in self.keys_to_use.iter().enumerate() {
            data_page.write_with_format(1, idx as u16, key.symbol_or_name(), &centered_bold)?;
        }

        let mut row = 0;
        for (result, buf) in self.data.iter() {
            data_page.write_with_format(row, 0, result.session_number, &centered_bold)?;
            let mut col = 1;
            row += 1;
            for key in self.keys_to_use.iter() {
                if !result.ksf.freq.iter().map(|(k, _)| k).contains(key) {
                    return Err(anyhow!(
                        "the key {} is not in the KSF for file {}",
                        key.symbol_or_name(),
                        buf.as_os_str().to_string_lossy()
                    ));
                } else {
                    data_page.write(row, col, *result.frequency_data.get(key).unwrap())?;
                    col += 1;
                }
            }
        }

        let chart_page = workbook.add_worksheet();
        chart_page.set_name("Graph")?;

        let chart_markers = [
            ChartMarkerType::Circle,
            ChartMarkerType::Square,
            ChartMarkerType::Triangle,
        ];

        let mut chart = Chart::new_line();
        let max_row = self.data.len() as u32;
        for idx in 0..self.keys_to_use.len() {
            let col = idx as u16;
            chart
                .add_series()
                .set_values(("Data", 1_u32, col, max_row, col))
                .set_name(("Data", 0, col))
                .set_marker(
                    ChartMarker::new()
                        .set_type(chart_markers[idx % 3])
                        .set_size(4),
                );
        }

        chart_page.insert_chart(2, 2, &chart)?;

        workbook.save(self.save_path.join("time_series.xlsx"))?;

        Ok(())
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
            self.data.clear();
            for buf in pathbufs {
                match SessionResults::from_file_path(buf.as_path()) {
                    Ok(data) => self.data.push((data, buf)),
                    Err(e) => windows_error_dialog(e),
                }
            }
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

            ui.label("Select Files From:");
            ui.directory_picker(
                &mut self.time_series.select_file_dialog,
                &self.time_series.select_path,
            );
            ui.add_space(10.0);

            ui.label("Save Graph To:");
            ui.directory_picker(
                &mut self.time_series.save_file_dialog,
                &self.time_series.save_path,
            );
            ui.add_space(15.0);

            self.time_series.key_picker(ui);

            ui.horizontal(|ui| {
                if ui.large_button("Select Data").clicked() {
                    self.time_series.select_file_dialog.pick_multiple();
                }

                if ui.small_button("clear").clicked() {
                    self.time_series.data.clear();
                }
            });

            egui::ScrollArea::vertical()
                .id_salt("time series")
                .show(ui, |ui| {
                    for (_, file_name) in self.time_series.data.iter() {
                        ui.monospace(file_name.file_name().unwrap().to_string_lossy());
                    }
                });

            if ui.large_green_button("Create Time Series").clicked() {
                quick_error!(self.time_series.create_time_series_graph());
            }

        });
    }
}
