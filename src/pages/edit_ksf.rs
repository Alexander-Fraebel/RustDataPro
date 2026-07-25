use crate::{
    app::{DataPro, KSF_FILE_NAME},
    data::{Data, Ksf, KsfData},
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
    preview: &mut String,
    save_finished: &mut bool,
) {
    if ui
        .add(egui::TextEdit::multiline(string).hint_text("M, Mand"))
        .changed()
    {
        *save_finished = false;
        preview.clear();
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

fn build_ksfs(ksfs: &mut KsfData, inputs: &(String, String, String)) -> Result<()> {
    let mut ksf = Ksf::default();

    for line in inputs.1.split("\n") {
        if !line.trim().is_empty() {
            let pair = parse_line(line)?;
            ksf.freq.push(pair);
        }
    }

    for line in inputs.2.split("\n") {
        if !line.trim().is_empty() {
            let pair = parse_line(line)?;
            ksf.freq.push(pair);
        }
    }

    ksfs.insert(inputs.0.clone(), ksf);
    Ok(())
}

#[derive(Default)]
pub struct NewKsf {
    user_input: Vec<(String, String, String)>,
    freq_preview: String,
    dura_preview: String,
    save_finished: bool,
}

impl NewKsf {
    fn save_file_to_path(&mut self, data: &Data, root_directory: &PathBuf) -> Result<()> {
        // if !self.ksf.all_unique() {
        //     return Err(anyhow::anyhow!(
        //         "ksf contains duplicate keys or duplicate descriptions"
        //     ));
        // }
        let p = Path::new(root_directory)
            .join(&format!("{}", data.client.id))
            .join(KSF_FILE_NAME);
        let mut writer = BufWriter::new(File::create_new(p)?);
        writer.write_all(data.ksfs.to_json()?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("Edit Keyboard Setup File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });
            ui.add_space(10.0);

            for (name, freq, dura) in app.new_ksf_page.user_input.iter_mut() {
                ui.horizontal(|ui| {
                    ui.monospace("Name");
                    ui.text_edit_singleline(name);
                });
                ui.horizontal(|ui| {
                    ui.monospace("Frequency Keys");
                    entry_row(
                        ui,
                        freq,
                        &mut app.new_ksf_page.freq_preview,
                        &mut app.new_ksf_page.save_finished,
                    )
                });
                ui.horizontal(|ui| {
                    ui.monospace("Duration Keys");
                    entry_row(
                        ui,
                        dura,
                        &mut app.new_ksf_page.dura_preview,
                        &mut app.new_ksf_page.save_finished,
                    )
                });
            }

            if ui.button("Add KSF").clicked() {
                app.new_ksf_page
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
                    for input in app.new_ksf_page.user_input.iter() {
                        if let Err(e) = build_ksfs(&mut temp_ksfs, input) {
                            windows_error_dialog(e);
                            write_succeeded = false;
                        }
                    }
                    if write_succeeded {
                        app.data.ksfs = temp_ksfs;
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
