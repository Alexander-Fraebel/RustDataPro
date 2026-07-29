use crate::{
    app::{DEFAULT_ROOT_DIRECTORY, DataPro, KSF_FILE_NAME},
    data::{ALLOWED_KEYS, Data, Ksf, KsfData},
    ui_elements::DataProUiElements,
    utils::{overwrite_file, windows_error_dialog},
};
use anyhow::Result;
use egui::{Color32, Key, RichText};
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use std::path::PathBuf;

fn parse_line(s: &str) -> Result<(Key, String)> {
    let (k, d) = match s.split_once(",") {
        Some((k, d)) => (k.trim(), d.trim()),
        None => {
            return Err(anyhow::anyhow!("no comma in line `{}`", s));
        }
    };
    let key = match Key::from_name(k) {
        Some(key) => {
            if !ALLOWED_KEYS.contains(&key) {
                return Err(anyhow::anyhow!(
                    "invalid key name `{}` in line `{}`",
                    key.symbol_or_name(),
                    s
                ));
            } else {
                key
            }
        }
        None => {
            return Err(anyhow::anyhow!("invalid key name `{}` in line `{}`", k, s));
        }
    };
    let desc = match d.contains(",") {
        true => {
            return Err(anyhow::anyhow!("too many commas in line `{}`", s));
        }
        false => d.to_string(),
    };
    Ok((key, desc))
}

fn entry_row(ui: &mut egui::Ui, string: &mut String, save_finished: &mut bool, hint: &str) {
    ui.label(hint);
    if ui.add(egui::TextEdit::multiline(string)).changed() {
        *save_finished = false;
    }
}

fn build_ksfs(ksfs: &mut KsfData, (name, freq, dura): &(String, String, String)) -> Result<()> {
    if !name.is_empty() {
        let mut ksf = Ksf::default();
        for line in freq.split("\n") {
            if !line.trim().is_empty() {
                let pair = parse_line(line)?;
                ksf.freq.push(pair);
            }
        }

        for line in dura.split("\n") {
            if !line.trim().is_empty() {
                let pair = parse_line(line)?;
                ksf.dura.push(pair);
            }
        }
        ksfs.insert(name.clone(), ksf);
    }
    Ok(())
}

fn ksf_scroller(app: &mut DataPro, ui: &mut egui::Ui) -> egui::scroll_area::ScrollAreaOutput<()> {
    if let Some(idx) = app.edit_ksfs.deleted_row {
        app.edit_ksfs.user_input.remove(idx);
        app.edit_ksfs.deleted_row = None;
    }
    ui.add_space(30.0);
    egui::ScrollArea::vertical()
        .min_scrolled_height(400.0)
        .id_salt("ksf_scroller")
        .show(ui, |ui| {
            for (n, (name, freq, dura)) in app.edit_ksfs.user_input.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            (220.0, 18.0),
                            egui::TextEdit::singleline(name)
                                .prefix(format!("{}) ", n + 1))
                                .hint_text("KSF Name"),
                        )
                        .changed()
                    {
                        app.edit_ksfs.save_finished = false;
                    }
                    if ui.small_button("delete").clicked() {
                        app.edit_ksfs.deleted_row = Some(n)
                    };
                });
                ui.add_space(5.0);

                entry_row(ui, freq, &mut app.edit_ksfs.save_finished, "Frequency Keys");
                ui.add_space(5.0);

                entry_row(ui, dura, &mut app.edit_ksfs.save_finished, "Duration Keys");
                ui.add_space(30.0);
            }
        })
}

fn edit_client_ksf(app: &mut DataPro, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ksf_scroller(app, ui);
        });
        ui.add_space(30.0);
        ui.vertical(|ui| {
            ui.add_space(30.0);
            if ui.button("Add KSF").clicked() {
                app.edit_ksfs
                    .user_input
                    .push((String::new(), String::new(), String::new()));
            }
            ui.add_space(10.0);

            // TODO: disable if invalid KSF writtens
            ui.add_enabled_ui(true, |ui| {
                if ui
                    .large_green_button("Save")
                    .on_disabled_hover_text("no file name provided")
                    .clicked()
                {
                    let mut write_succeeded = true;
                    let mut temp_ksfs = KsfData::default();
                    for input in app.edit_ksfs.user_input.iter() {
                        if let Err(e) = build_ksfs(&mut temp_ksfs, input) {
                            windows_error_dialog(e);
                            write_succeeded = false;
                        }
                    }
                    if write_succeeded {
                        app.data.ksfs = temp_ksfs;
                        match app.overwrite_ksf_data() {
                            Ok(_) => app.edit_ksfs.save_finished = true,
                            Err(e) => {
                                windows_error_dialog(e);
                                app.edit_ksfs.save_finished = false;
                            }
                        }
                    }
                }
            });

            if ui.large_red_button("Return").clicked() {
                app.edit_ksfs.save_finished = false;
                app.display_info.go_to_prep_session();
            }

            if app.edit_ksfs.save_finished {
                ui.monospace(
                    RichText::new("KSF Updated!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    });
}

fn new_ksf_creator(app: &mut DataPro, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ksf_scroller(app, ui);
        });
        ui.add_space(30.0);
        ui.vertical(|ui| {
            ui.add_space(30.0);
            if ui.button("Add KSF").clicked() {
                app.edit_ksfs.user_input.push(Default::default());
            }
            ui.add_space(10.0);

            if ui.large_green_button("Save").clicked() {
                let mut write_succeeded = true;
                let mut temp_ksf_data = KsfData::default();
                for input in app.edit_ksfs.user_input.iter() {
                    if let Err(e) = build_ksfs(&mut temp_ksf_data, input) {
                        windows_error_dialog(e);
                        write_succeeded = false;
                    }
                }
                let mut output_json = String::new();
                match temp_ksf_data.to_json() {
                    Ok(json) => output_json = json,
                    Err(e) => {
                        windows_error_dialog(e);
                        write_succeeded = false;
                    }
                }
                if write_succeeded {
                    match overwrite_file(
                        Ok(app.edit_ksfs.new_ksf_path.join(KSF_FILE_NAME)),
                        &output_json,
                    ) {
                        Ok(_) => app.edit_ksfs.save_finished = true,
                        Err(e) => {
                            windows_error_dialog(e);
                            app.edit_ksfs.save_finished = false;
                        }
                    }
                }
            }

            if ui.large_red_button("Return").clicked() {
                app.edit_ksfs.save_finished = false;
                app.display_info.go_to_prep_session();
            }

            if app.edit_ksfs.save_finished {
                ui.monospace(
                    RichText::new("KSF Created!")
                        .heading()
                        .color(Color32::GREEN),
                );
            }
        });
    });
}

#[derive(Default)]
pub struct EditKsfData {
    pub user_input: Vec<(String, String, String)>,
    pub save_finished: bool,
    pub deleted_row: Option<usize>,
    pub file_dialog: FileDialog,
    pub new_ksf_path: PathBuf,
}

impl EditKsfData {
    pub fn prepare(&mut self, data: &Data) {
        self.user_input.clear();

        // If there is a client loaded rebuild the UI with the client information
        if data.client_loaded() {
            for (name, ksf) in data.ksfs.iter() {
                let (freq, dura) = ksf.pairs();
                self.user_input.push((
                    name.to_string(),
                    freq.map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                        .join("\n"),
                    dura.map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                        .join("\n"),
                ));
            }
        } else {
            // If there is no client loaded create a UI with a single empty region to start with
            self.file_dialog = FileDialog::new().initial_directory(DEFAULT_ROOT_DIRECTORY.into());
            self.user_input.push(Default::default());
            self.new_ksf_path = DEFAULT_ROOT_DIRECTORY.into();
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        app.edit_ksfs.file_dialog.update(ui.ctx());
        if let Some(pathbuf) = app.edit_ksfs.file_dialog.take_picked() {
            app.edit_ksfs.new_ksf_path = pathbuf;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Keyboard Setup File for ");
                ui.client_picker(app, "ksf_page_client_picker");
            });

            ui.label("If a client is selected this page will automatically udpate\nthe KSF for that client. If no client is selected you may\nsave this KSF created here to the directory below.");
            ui.add_space(10.0);
            
            ui.add_enabled_ui(!app.data.client_loaded(), |ui| {
                ui.label("Save File To:");
                ui.directory_picker(&mut app.edit_ksfs.file_dialog, &app.edit_ksfs.new_ksf_path);
            });
            ui.add_space(10.0);

            if app.data.client_loaded() {
                edit_client_ksf(app, ui)
            } else {
                new_ksf_creator(app, ui)
            }
        });
    }
}
