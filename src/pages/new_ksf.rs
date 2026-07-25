use crate::{
    app::DataPro,
    data::{Data, KsfData},
    utils::{DataProUiElements, windows_error_dialog},
};
use anyhow::{Context, Result};
use egui::{Color32, Key, RichText};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

const ALLOWED_KEYS: [Key; 36] = [
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
];

fn parse_line(s: &str) -> Result<(Key, String)> {
    let (k, d) = s.split_once(",").context("no comma")?;
    let key = match Key::from_name(k.trim()) {
        Some(key) => {
            if !ALLOWED_KEYS.contains(&key) {
                return Err(anyhow::anyhow!("invalid key name"));
            } else {
                key
            }
        }
        None => {
            return Err(anyhow::anyhow!("invalid key name"));
        }
    };
    let desc = match d.contains(",") {
        true => {
            return Err(anyhow::anyhow!("too many commas"));
        }
        false => d.trim().to_string(),
    };
    Ok((key, desc))
}

fn entry_row(
    ui: &mut egui::Ui,
    string: &mut String,
    vector: &mut Vec<(Key, String)>,
    preview: &mut String,
    save_finished: &mut bool,
) {
    if ui
        .add(egui::TextEdit::multiline(string).hint_text("M, Mand"))
        .changed()
    {
        *save_finished = false;
        preview.clear();
        vector.clear();
        for line in string.split("\n") {
            if !line.is_empty() {
                if let Err(e) = parse_line(line) {
                    preview.push_str(&format!("{}\n", e));
                } else {
                    preview.push('\n');
                }
            } else {
                preview.push('\n');
            }
        }
        // remove trailing newline
        preview.pop();
    }

    ui.add(
        egui::TextEdit::multiline(preview)
            .background_color(ui.visuals().window_fill)
            .interactive(false),
    );
}

#[derive(Default)]
pub struct NewKsf {
    ksf: KsfData,
    file_name: String,
    freq_string: String,
    dura_string: String,
    freq_preview: String,
    dura_preview: String,
    save_finished: bool,
}

impl NewKsf {
    fn save_file_to_path(&mut self, data: &Data, root_directory: &PathBuf) -> Result<()> {
        if !self.ksf.all_unique() {
            return Err(anyhow::anyhow!(
                "ksf contains duplicate keys or duplicate descriptions"
            ));
        }
        let p = Path::new(root_directory)
            .join(&format!("{}", data.client.id))
            .join(&format!("{}.txt", self.file_name));
        let mut writer = BufWriter::new(File::create_new(p)?);
        writer.write_all(self.ksf.to_json()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("New Keyboard Setup File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
            ui.add_space(10.0);

            ui.label("Name");
            ui.text_edit_singleline(&mut app.new_ksf_page.file_name);
            ui.add_space(10.0);

            egui::Grid::new("new_ksf_info_grid")
                .min_col_width(150.0)
                .show(ui, |ui| {
                    ui.monospace("Frequency Keys");
                    entry_row(
                        ui,
                        &mut app.new_ksf_page.freq_string,
                        &mut app.new_ksf_page.ksf.frequency,
                        &mut app.new_ksf_page.freq_preview,
                        &mut app.new_ksf_page.save_finished,
                    );
                    ui.end_row();

                    ui.monospace("Duration Keys");
                    entry_row(
                        ui,
                        &mut app.new_ksf_page.dura_string,
                        &mut app.new_ksf_page.ksf.duration,
                        &mut app.new_ksf_page.dura_preview,
                        &mut app.new_ksf_page.save_finished,
                    );
                    ui.end_row();
                });

            ui.add_enabled_ui(!app.new_ksf_page.file_name.is_empty(), |ui| {
                if ui
                    .large_green_button("Save")
                    .on_disabled_hover_text("no file name provided")
                    .clicked()
                {
                    match app
                        .new_ksf_page
                        .save_file_to_path(&app.data, &app.root_directory)
                    {
                        Ok(_) => app.new_ksf_page.save_finished = true,
                        Err(e) => {
                            windows_error_dialog(e);
                            app.new_ksf_page.save_finished = false;
                        }
                    }
                }
            });

            if ui.large_red_button("Return").clicked() {
                app.new_ksf_page.save_finished = false;
                app.display_info.go_to_prep_session();
            }

            if app.new_ksf_page.save_finished {
                ui.monospace(RichText::new("KSF Saved!").heading().color(Color32::GREEN));
            }
        });
    }
}
