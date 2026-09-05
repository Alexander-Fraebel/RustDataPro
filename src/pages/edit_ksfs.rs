use crate::{
    app::DataPro,
    data::{ALLOWED_KSF_KEYS, Ksf, KsfsData},
    ui_elements::DataProUiElements,
    utils::{are_you_sure_dialog, overwrite_file, windows_error_dialog},
};
use anyhow::Result;
use egui::{Key, RichText, TextStyle};
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
            if !ALLOWED_KSF_KEYS.contains(&key) {
                return Err(anyhow::anyhow!(
                    "invalid key name `{}` in line `{}`\nonly letters and numbers are allowed",
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

fn entry_row(ui: &mut egui::Ui, string: &mut String, changes_made: &mut bool, label: &str) {
    ui.label(label);
    if ui
        .add(
            egui::TextEdit::multiline(string)
                .hint_text(RichText::from("A, Description\nB, Description\n...").monospace())
                .font(TextStyle::Monospace),
        )
        .changed()
    {
        *changes_made = true;
    }
}

fn ksf_scroller(app: &mut DataPro, ui: &mut egui::Ui) -> egui::scroll_area::ScrollAreaOutput<()> {
    if let Some(idx) = app.edit_ksfs.deleted_row {
        app.edit_ksfs.user_input.remove(idx);
        app.edit_ksfs.deleted_row = None;
        if app.edit_ksfs.user_input.is_empty() {
            app.edit_ksfs.user_input.push(KsfMaker::default());
            app.data.session.chosen_ksf_name.clear();
        }
    }
    ui.style_mut().spacing.scroll = egui::style::ScrollStyle::solid();
    ui.add_space(10.0);
    egui::ScrollArea::vertical()
        .min_scrolled_height(450.0)
        .id_salt("ksf_scroller")
        .show(ui, |ui| {
            for (n, ksf_maker) in app.edit_ksfs.user_input.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            (220.0, 18.0),
                            egui::TextEdit::singleline(&mut ksf_maker.name).hint_text("KSF Name"),
                        )
                        .changed()
                    {
                        app.edit_ksfs.changes_made = true;
                    }
                    if ui.small_button("delete").clicked() {
                        if are_you_sure_dialog("Delete this KSF?") {
                            app.edit_ksfs.deleted_row = Some(n)
                        }
                    };
                });
                ui.add_space(5.0);

                entry_row(
                    ui,
                    &mut ksf_maker.freq,
                    &mut app.edit_ksfs.changes_made,
                    "Frequency Keys",
                );
                ui.add_space(5.0);

                entry_row(
                    ui,
                    &mut ksf_maker.dura,
                    &mut app.edit_ksfs.changes_made,
                    "Duration Keys",
                );
                ui.add_space(30.0);
            }
        })
}

fn ksf_controller(app: &mut DataPro, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ksf_scroller(app, ui);
        });
        ui.add_space(30.0);
        ui.vertical(|ui| {
            ui.add_space(10.0);
            if ui.button("Add KSF").clicked() {
                app.edit_ksfs.user_input.push(KsfMaker::default());
            }
            ui.add_space(10.0);
            if ui.large_green_button("SAVE").clicked() {
                app.save_and_reload_ksfs();
            }
        });
    });
}

#[derive(Default)]
pub struct KsfMaker {
    name: String,
    freq: String,
    dura: String,
}

impl KsfMaker {
    fn from_ksf(name: &str, ksf: &Ksf) -> Self {
        Self {
            name: name.to_string(),
            freq: ksf
                .freq
                .iter()
                .map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                .join("\n"),
            dura: ksf
                .dura
                .iter()
                .map(|(k, d)| format!("{}, {}", k.symbol_or_name(), d))
                .join("\n"),
        }
    }

    fn into_ksf(&self) -> Result<Ksf> {
        let mut freq = Vec::new();
        for line in self.freq.split("\n") {
            if !line.trim().is_empty() {
                freq.push(parse_line(line)?);
            }
        }
        let mut dura = Vec::new();
        for line in self.dura.split("\n") {
            if !line.trim().is_empty() {
                dura.push(parse_line(line)?);
            }
        }
        Ok(Ksf { freq, dura })
    }
}

#[derive(Default)]
pub struct EditKsfData {
    pub user_input: Vec<KsfMaker>,
    pub changes_made: bool, // default is false
    pub deleted_row: Option<usize>,
    pub file_dialog: FileDialog,
    pub save_path: PathBuf,
}

impl DataPro {
    pub fn prepare_edit_ksf_page(&mut self) {
        // Reset to make sure we don't have fields fill with dirty information.
        *self = Self::default();

        // Load the path information
        // This will be default information automatically if anything has gone wrong
        self.edit_ksfs.save_path = self.path_to_ksf_data();
        self.edit_ksfs.file_dialog = FileDialog::new().initial_directory(self.path_to_ksf_data());

        // If there is a client loaded rebuild the UI with the client information
        if self.data.client_loaded() {
            for (name, ksf) in self.data.ksfs.iter() {
                self.edit_ksfs
                    .user_input
                    .push(KsfMaker::from_ksf(name, ksf));
            }
        }
        // Ensure the UI is not empty
        if self.edit_ksfs.user_input.is_empty() {
            self.edit_ksfs.user_input.push(Default::default());
        }
    }

    pub fn view_edit_ksf_page(&mut self, ui: &mut egui::Ui) {
        self.edit_ksfs.file_dialog.update(ui.ctx());
        if let Some(pathbuf) = self.edit_ksfs.file_dialog.take_picked() {
            self.edit_ksfs.save_path = pathbuf;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Keyboard Setup Files For");
            self.client_picker(ui);
            ui.add_space(15.0);

            ui.label("Save To:");
            ui.directory_picker(&mut self.edit_ksfs.file_dialog, &self.edit_ksfs.save_path);
            ui.add_space(10.0);

            ksf_controller(self, ui)
        });
    }

    pub fn save_and_reload_ksfs(&mut self) {
        let mut write_succeeded = true;
        let mut temp_ksf_data = KsfsData::default();

        // Check if each KSF builds
        for input in self.edit_ksfs.user_input.iter() {
            match input.into_ksf() {
                Ok(ksf) => {
                    temp_ksf_data.insert(input.name.clone(), ksf);
                }
                Err(e) => {
                    windows_error_dialog(e);
                    write_succeeded = false;
                }
            }
        }
        // Check if each KSF is valid
        if let Err(e) = temp_ksf_data.all_keys_unique() {
            windows_error_dialog(e);
            write_succeeded = false;
        }
        // Check if the JSON builds
        let mut output_json = String::new();
        match temp_ksf_data.to_json() {
            Ok(json) => output_json = json,
            Err(e) => {
                windows_error_dialog(e);
                write_succeeded = false;
            }
        }
        // Write the file
        if write_succeeded {
            match overwrite_file(Ok(self.edit_ksfs.save_path.clone()), &output_json) {
                Ok(_) => {
                    if !self.data.ksf_loaded() {
                        self.load_ksf(&self.path_to_ksf_data());
                    } else {
                        self.data.ksfs = temp_ksf_data;
                    }
                }
                Err(e) => {
                    windows_error_dialog(e.context("error while saving KsfData"));
                    self.edit_ksfs.changes_made = false;
                }
            }
        }
    }
}
