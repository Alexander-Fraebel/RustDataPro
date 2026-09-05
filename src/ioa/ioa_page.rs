use crate::{
    app::DataPro,
    data::{DataCollectionType, IoaData, OutputData},
    ioa::{
        calculations::{single_pair_interval_ioa, single_pair_total_ratio_ioa},
        excel_output::save_excel_workbook,
    },
    utils::{DataProUiElements, quick_file_name, time_stamp, windows_error_dialog},
};
use anyhow::{Context, Result};
use egui::{Color32, RichText, Ui};
use egui_file_dialog::FileDialog;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub struct IoaPage {
    pub prim_data: Vec<(OutputData, PathBuf)>,
    pub reli_data: Vec<(OutputData, PathBuf)>,
    pub ioa_finished: bool,
    pub strict: bool,
    pub none_val: f32,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl Default for IoaPage {
    fn default() -> Self {
        Self {
            prim_data: Vec::new(),
            reli_data: Vec::new(),
            ioa_finished: false,
            strict: true,
            none_val: f32::NAN,
            select_file_dialog: FileDialog::new(),
            select_path: PathBuf::default(),
            save_file_dialog: FileDialog::new(),
            save_path: PathBuf::default(),
        }
    }
}

impl IoaPage {
    pub fn prepare(&mut self, select_path: PathBuf, save_new_path: PathBuf) {
        *self = Self::default();
        self.select_path = select_path.clone();
        self.select_file_dialog = FileDialog::new().initial_directory(select_path.clone());
        self.save_path = save_new_path.clone();
        self.save_file_dialog = FileDialog::new().initial_directory(save_new_path.clone());
    }

    fn interval_ioa(&self, ioa_data: &mut IoaData) {
        for ((p, _), (r, _)) in self.prim_data.iter().zip(self.reli_data.iter()) {
            let max_time = if p.session_duration >= r.session_duration {
                p.session_duration
            } else {
                r.session_duration
            };
            let (freq, dura) = p.ksf.keys();
            for key in freq.chain(dura) {
                // 10 Second Interval-by-Interval IOA
                let r10 = single_pair_interval_ioa(
                    max_time,
                    10.0,
                    *key,
                    &p.timeline,
                    &r.timeline,
                    self.strict,
                )
                .unwrap_or(self.none_val);
                ioa_data.ten_sec_interval[key] += r10;

                // 60 Second Interval-by-Interval IOA
                let r60 = single_pair_interval_ioa(
                    max_time,
                    60.0,
                    *key,
                    &p.timeline,
                    &r.timeline,
                    self.strict,
                )
                .unwrap_or(self.none_val);
                ioa_data.sixty_sec_interval[key] += r60;
            }
        }
    }

    fn frequency_ioa(&self, ioa_data: &mut IoaData) -> Result<()> {
        for ((p, _), (r, _)) in self.prim_data.iter().zip(self.reli_data.iter()) {
            for (key, _desc) in p.ksf.freq.iter() {
                // Total Count IOA
                let primary_count =
                    *p.frequency.get(key).context("missing primary duration")? as f32; // conversion of u32 to f32 is valid so long as count is below about 16 million, so it is not checked
                let reli_count = *r.frequency.get(key).context("missing reli duration")? as f32;
                ioa_data.total_count[key] +=
                    single_pair_total_ratio_ioa(primary_count, reli_count).unwrap_or(self.none_val);
            }
        }
        Ok(())
    }

    fn duration_ioa(&self, ioa_data: &mut IoaData) -> Result<()> {
        for ((p, _), (r, _)) in self.prim_data.iter().zip(self.reli_data.iter()) {
            for (key, _desc) in p.ksf.dura.iter() {
                // Total Duration IOA
                let primary_dur = p.duration.get(key).context("missing primary duration")?.1;
                let reli_dur = r.duration.get(key).context("missing reli duration")?.1;
                ioa_data.total_duration[key] +=
                    single_pair_total_ratio_ioa(primary_dur, reli_dur).unwrap_or(self.none_val);

                // Total Count IOA (onset and offset of duration keys)
                let primary_count =
                    p.duration.get(key).context("missing primary duration")?.0 as f32;
                let reli_count = r.duration.get(key).context("missing reli duration")?.0 as f32;
                ioa_data.total_count[key] +=
                    single_pair_total_ratio_ioa(primary_count, reli_count).unwrap_or(self.none_val);
            }
        }
        Ok(())
    }

    pub fn calculate_ioa(&mut self, ioa_directory: &PathBuf) -> Result<()> {
        let mut ioa_data = IoaData::from_ksf(&self.prim_data[0].0.ksf);

        self.interval_ioa(&mut ioa_data);
        self.frequency_ioa(&mut ioa_data)?;
        self.duration_ioa(&mut ioa_data)?;

        ioa_data.normalize(self.prim_data.len() as f32)?;
        let path = Path::new(ioa_directory)
            .join(format!("reliability_{}", time_stamp()))
            .to_string_lossy()
            .to_string();

        save_excel_workbook(&ioa_data, &path, &self.prim_data, &self.reli_data)?;

        let mut writer = BufWriter::new(File::create(&format!("{}.txt", path))?);
        writer.write_all(ioa_data.to_json()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }
}

impl DataPro {
    pub fn view_ioa(&mut self, ui: &mut Ui) {
        self.ioa_page.select_file_dialog.update(ui.ctx());
        if let Some(pathbuf) = self.ioa_page.select_file_dialog.take_picked() {
            self.ioa_page
                .prepare(pathbuf, self.ioa_page.save_path.clone());
        }

        self.ioa_page.save_file_dialog.update(ui.ctx());
        if let Some(pathbuf) = self.ioa_page.save_file_dialog.take_picked() {
            self.ioa_page
                .prepare(self.ioa_page.select_path.clone(), pathbuf);
        }
        if let Some(bufs) = self.ioa_page.select_file_dialog.take_picked_multiple() {
            self.ioa_page.prepare(
                self.path_to_session_records_dir(),
                self.path_to_ioa_data_dir(),
            );
            // Simultaneously parse and filter the input files.
            for buf in bufs {
                match OutputData::from_file_path(buf.as_path()) {
                    Ok(data) => match data.session.data_collection_type {
                        DataCollectionType::Primary => self.ioa_page.prim_data.push((data, buf)),
                        DataCollectionType::Reliability => {
                            self.ioa_page.reli_data.push((data, buf))
                        }
                    },
                    Err(_) => (),
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Calculate IOA");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("Select Files From:");
            ui.directory_picker(
                &mut self.ioa_page.select_file_dialog,
                &self.ioa_page.select_path,
            );
            ui.add_space(10.0);

            ui.label("Save IOA To:");
            ui.directory_picker(
                &mut self.ioa_page.save_file_dialog,
                &self.ioa_page.save_path,
            );
            ui.add_space(15.0);

            if ui.large_button("Select Data").clicked() {
                self.ioa_page.select_file_dialog.pick_multiple();
            }
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.add_sized([110.0, 20.0], egui::Label::new("Primary Data"));
                        egui::containers::ScrollArea::vertical()
                            .content_margin(10.0)
                            .id_salt("prim_info_area")
                            .show(ui, |ui| {
                                for (_, path) in self.ioa_page.prim_data.iter() {
                                    ui.strong(format!("{}", quick_file_name(&path)));
                                }
                            });
                    })
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.add_sized([110.0, 20.0], egui::Label::new("Reliability Data"));
                        egui::containers::ScrollArea::vertical()
                            .id_salt("reli_info_area")
                            .content_margin(10.0)
                            .show(ui, |ui| {
                                for (_, path) in self.ioa_page.reli_data.iter() {
                                    ui.strong(format!("{}", quick_file_name(&path)));
                                }
                            });
                    });
                });
            });
            ui.add_space(20.0);

            if ui.large_green_button("Calculate IOA").clicked() {
                self.save_new_ioa_data()
                    .unwrap_or_else(|e| windows_error_dialog(e))
            }
            ui.add_space(5.0);

            if self.ioa_page.ioa_finished {
                ui.monospace(
                    RichText::new("IOA Calculated and Saved!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    }
}
