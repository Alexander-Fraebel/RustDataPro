use crate::{
    app::DataPro,
    data::{DataType, IoaData, OutputData},
    ioa::{
        calculations::{single_pair_interval_ioa, single_pair_total_ratio_ioa},
        excel_output::save_excel_workbook,
        validate_files::validate_files,
    },
    utils::{DataProUiElements, quick_file_name, time_stamp},
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
    pub file_dialog: FileDialog,
    pub prim_data: Vec<(OutputData, PathBuf)>,
    pub reli_data: Vec<(OutputData, PathBuf)>,
    pub error: String,
    pub ioa_finished: bool,
    pub strict: bool,
    pub none_val: f32,
}

impl Default for IoaPage {
    fn default() -> Self {
        Self {
            file_dialog: Default::default(),
            prim_data: Vec::new(),
            reli_data: Vec::new(),
            error: String::new(),
            ioa_finished: false,
            strict: true,
            none_val: f32::NAN,
        }
    }
}

impl IoaPage {
    fn push_error(&mut self, text: &str) {
        if self.error.is_empty() {
            self.error.push_str(text);
        } else {
            self.error.push('\n');
            self.error.push_str(text);
        }
    }

    fn clear(&mut self) {
        self.prim_data.clear();
        self.reli_data.clear();
        self.error.clear();
        self.ioa_finished = false;
    }

    fn interval_ioa(&self, ioa_data: &mut IoaData) {
        for ((p, _), (r, _)) in self.prim_data.iter().zip(self.reli_data.iter()) {
            let max_time = if p.session_duration >= r.session_duration {
                p.session_duration
            } else {
                r.session_duration
            };
            for key in p.ksf.keys() {
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
            for (key, _desc) in p.ksf.frequency.iter() {
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
            for (key, _desc) in p.ksf.duration.iter() {
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

    fn calculate_ioa(&mut self, ioa_directory: &PathBuf) -> Result<()> {
        let mut ioa_data = IoaData::from_ksf(&self.prim_data[0].0.ksf);

        self.interval_ioa(&mut ioa_data);
        self.frequency_ioa(&mut ioa_data)?;
        self.duration_ioa(&mut ioa_data)?;

        ioa_data.finalize(self.prim_data.len() as f32);
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

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.ioa_page.file_dialog.update(ui.ctx());
        if let Some(bufs) = app.ioa_page.file_dialog.take_picked_multiple() {
            app.ioa_page.clear();
            // Simultaneously parse and filter the input files.
            for buf in bufs {
                match OutputData::from_file(buf.as_path()) {
                    Ok(data) => match data.session.data_type {
                        DataType::Primary => app.ioa_page.prim_data.push((data, buf)),
                        DataType::Reliability => app.ioa_page.reli_data.push((data, buf)),
                    },
                    Err(_) => (), //app.ioa_page.push_error(&e.to_string()),
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("Calculate IOA for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
            ui.add_space(10.0);

            if ui.large_button("Select Data").clicked() {
                app.ioa_page.file_dialog.pick_multiple();
            }
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Primary Data");
                        egui::containers::ScrollArea::vertical()
                            .id_salt("prim_info_area")
                            .show(ui, |ui| {
                                for (_, path) in app.ioa_page.prim_data.iter() {
                                    ui.strong(format!("{}", quick_file_name(&path)));
                                }
                            });
                    })
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Reliability Data");
                        egui::containers::ScrollArea::vertical()
                            .id_salt("reli_info_area")
                            .show(ui, |ui| {
                                for (_, path) in app.ioa_page.reli_data.iter() {
                                    ui.strong(format!("{}", quick_file_name(&path)));
                                }
                            });
                    });
                });
            });
            ui.add_space(20.0);

            ui.add_enabled_ui(app.data.client_loaded(), |ui| {
                if ui
                    .large_green_button("Calculate IOA")
                    .on_disabled_hover_text("no client selected")
                    .clicked()
                {
                    if !app.ioa_page.ioa_finished {
                        match validate_files(&app.ioa_page.prim_data, &app.ioa_page.reli_data) {
                            Ok(_) => match app.ioa_page.calculate_ioa(
                                &app.client_ioa_data_path().expect("ERROR REACHING IOA DATA"),
                            ) {
                                Ok(_) => {
                                    app.ioa_page.error.clear();
                                    app.ioa_page.ioa_finished = true;
                                }
                                Err(e) => app.ioa_page.push_error(&e.to_string()),
                            },
                            Err(e) => app.ioa_page.push_error(&e.to_string()),
                        }
                    }
                }
            });

            ui.add_space(5.0);

            if ui.large_red_button("Return").clicked() {
                app.ioa_page.clear();
                app.display_info.go_to_prep_session();
            }
            ui.add_space(5.0);

            ui.strong(&app.ioa_page.error);
            ui.add_space(5.0);

            if app.ioa_page.ioa_finished {
                ui.monospace(
                    RichText::new("IOA Calculated and Saved!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    }
}
