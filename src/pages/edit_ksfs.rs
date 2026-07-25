use crate::{
    app::DataPro,
    data::{Ksf, KsfData},
    utils::{DataProUiElements, windows_error_dialog},
};
use anyhow::Result;
use egui::{Color32, Key, RichText};

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
    if ui
        .add(egui::TextEdit::multiline(string).hint_text(hint))
        .changed()
    {
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

#[derive(Default)]
pub struct EditKsfData {
    pub user_input: Vec<(String, String, String)>,
    pub save_finished: bool,
}

impl EditKsfData {
    pub fn view(app: &mut DataPro, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.heading("Edit Keyboard Setup File for Client ");
                ui.add(egui::Label::new(
                    egui::RichText::new(&app.data.client.id).heading().strong(),
                ));
            });

            for (name, freq, dura) in app.edit_ksfs.user_input.iter_mut() {
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::TextEdit::singleline(name).hint_text("KSF Name"))
                        .changed()
                    {
                        app.edit_ksfs.save_finished = false;
                    }
                });
                ui.add_space(5.0);

                entry_row(ui, freq, &mut app.edit_ksfs.save_finished, "Frequency Keys");
                ui.add_space(5.0);

                entry_row(ui, dura, &mut app.edit_ksfs.save_finished, "Duration Keys");
            }

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
                ui.monospace(RichText::new("KSF Update!").heading().color(Color32::GREEN));
            }
        });
    }
}
