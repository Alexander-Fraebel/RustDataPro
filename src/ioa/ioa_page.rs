use super::IoaDataMap;
use crate::{
    app::DataPro,
    data::{IoaData, SessionResults},
    ioa::{
        calculations::{single_pair_interval_ioa, single_pair_total_ratio_ioa},
        excel_output::save_excel_workbook,
        quick_file_name,
    },
    utils::{time_stamp, ui_elements::DataProUiElements, windows_error_dialog},
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
    pub ioa_finished: bool,
    pub strict: bool, // if true ignore intervals with no data from either recorder when calculating reliability, defaults to true
    pub none_val: f32,
    pub select_file_dialog: FileDialog,
    pub select_path: PathBuf,
    pub save_file_dialog: FileDialog,
    pub save_path: PathBuf,
    pub data: IoaDataMap,
}

impl Default for IoaPage {
    fn default() -> Self {
        Self {
            ioa_finished: false,
            strict: true,
            none_val: f32::NAN,
            select_file_dialog: FileDialog::new(),
            select_path: PathBuf::default(),
            save_file_dialog: FileDialog::new(),
            save_path: PathBuf::default(),
            data: IoaDataMap::default(),
        }
    }
}

impl IoaPage {
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

    fn interval_ioa(
        &self,
        ioa_data: &mut IoaData,
        simplified: &Vec<(&(SessionResults, PathBuf), &(SessionResults, PathBuf))>,
    ) -> Result<()> {
        for ((p, _), (r, _)) in simplified {
            let max_time = if p.total_time >= r.total_time {
                p.total_time
            } else {
                r.total_time
            };
            let (freq, dura) = p.ksf.freq_and_dura_keys();
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
        Ok(())
    }

    fn frequency_ioa(
        &self,
        ioa_data: &mut IoaData,
        simplified: &Vec<(&(SessionResults, PathBuf), &(SessionResults, PathBuf))>,
    ) -> Result<()> {
        for ((p, _), (r, _)) in simplified {
            for (key, _desc) in p.ksf.freq.iter() {
                // Total Count IOA
                let primary_count =
                    *p.frequency_data
                        .get(key)
                        .context("missing primary duration")? as f32; // conversion of u32 to f32 is valid so long as count is below about 16 million, so it is not checked
                let reli_count =
                    *r.frequency_data.get(key).context("missing reli duration")? as f32;
                ioa_data.total_count[key] +=
                    single_pair_total_ratio_ioa(primary_count, reli_count).unwrap_or(self.none_val);
            }
        }
        Ok(())
    }

    fn duration_ioa(
        &self,
        ioa_data: &mut IoaData,
        simplified: &Vec<(&(SessionResults, PathBuf), &(SessionResults, PathBuf))>,
    ) -> Result<()> {
        for ((p, _), (r, _)) in simplified {
            for (key, _desc) in p.ksf.dura.iter() {
                // Total Duration IOA
                let primary_dur = p
                    .duration_data
                    .get(key)
                    .context("missing primary duration")?
                    .1;
                let reli_dur = r.duration_data.get(key).context("missing reli duration")?.1;
                ioa_data.total_duration[key] +=
                    single_pair_total_ratio_ioa(primary_dur, reli_dur).unwrap_or(self.none_val);

                // Total Count IOA (onset and offset of duration keys)
                let primary_count = p
                    .duration_data
                    .get(key)
                    .context("missing primary duration")?
                    .0 as f32;
                let reli_count =
                    r.duration_data.get(key).context("missing reli duration")?.0 as f32;
                ioa_data.total_count[key] +=
                    single_pair_total_ratio_ioa(primary_count, reli_count).unwrap_or(self.none_val);
            }
        }
        Ok(())
    }

    pub fn calculate_ioa(&mut self, ioa_directory: &PathBuf) -> Result<()> {
        let simplified = self.data.validated()?;

        let ksf = &simplified[0].0.0.ksf;
        let mut ioa_data = IoaData::from_ksf(ksf);

        self.interval_ioa(&mut ioa_data, &simplified)?;
        self.frequency_ioa(&mut ioa_data, &simplified)?;
        self.duration_ioa(&mut ioa_data, &simplified)?;

        ioa_data.normalize(simplified.len() as f32)?;
        let path = Path::new(ioa_directory)
            .join(format!("reliability_{}", time_stamp()))
            .to_string_lossy()
            .to_string();

        save_excel_workbook(&ioa_data, &path, &simplified)?;

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
                match SessionResults::from_file_path(buf.as_path()) {
                    Ok(results) => {
                        crate::quick_error!(self.ioa_page.data.insert(results, buf))
                    }
                    Err(e) => windows_error_dialog(e),
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Calculate IOA for");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                if ui.large_button("Select Data").clicked() {
                    self.ioa_page.select_file_dialog.pick_multiple();
                }
                ui.add_space(5.0);
                if ui.large_green_button("Calculate IOA").clicked() {
                    self.save_new_ioa_data()
                        .unwrap_or_else(|e| windows_error_dialog(e))
                }
            });
            ui.add_space(5.0);

            ui.group(|ui| {
                egui::containers::ScrollArea::vertical()
                    .content_margin(15.0)
                    .min_scrolled_height(200.0)
                    .max_height(200.0)
                    .id_salt("prim_info_area")
                    .show(ui, |ui| {
                        egui::Grid::new("compare_prim_and_reli_grid")
                            .min_col_width(40.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Primary Data").monospace().underline());
                                ui.monospace("");
                                ui.monospace(
                                    RichText::new("Reliability Data").monospace().underline(),
                                );
                                ui.end_row();
                                for (_, (p, r)) in self.ioa_page.data.0.iter() {
                                    match (p, r) {
                                        (None, None) => todo!(),
                                        (None, Some((_, path))) => {
                                            let name = RichText::new(format!(
                                                "{}",
                                                quick_file_name(&path)
                                            ))
                                            .monospace();
                                            ui.monospace(
                                                RichText::new("Missing!").color(Color32::RED),
                                            );
                                            ui.monospace("");
                                            ui.label(name.color(Color32::RED));
                                        }
                                        (Some((_, path)), None) => {
                                            let name = RichText::new(format!(
                                                "{}",
                                                quick_file_name(&path)
                                            ))
                                            .monospace();
                                            ui.label(name.color(Color32::RED));
                                            ui.monospace("");
                                            ui.monospace(
                                                RichText::new("Missing!").color(Color32::RED),
                                            );
                                        }
                                        (Some((p_res, p_path)), Some((r_res, r_path))) => {
                                            let p_name = RichText::new(format!(
                                                "{}",
                                                quick_file_name(&p_path)
                                            ))
                                            .monospace();
                                            let r_name = RichText::new(format!(
                                                "{}",
                                                quick_file_name(&r_path)
                                            ))
                                            .monospace();

                                            if p_res.complement_of(&r_res) {
                                                ui.label(p_name);
                                                ui.monospace("");
                                                ui.label(r_name);
                                            } else {
                                                ui.label(p_name.color(Color32::RED));
                                                ui.monospace("");
                                                ui.label(r_name.color(Color32::RED));
                                            }
                                        }
                                    }

                                    ui.end_row();
                                }
                            });
                    });
            });
            ui.add_space(20.0);

            ui.label("Files are highlighed in red if they cannot be compared for some reason. If both files are present and the file names match open the file in Notepad to check that the KSF, Client ID, Assessment, and Condition are all the same.");
            ui.add_space(20.0);

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
