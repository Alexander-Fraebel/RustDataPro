use crate::{
    app::DataPro,
    ui_elements::DataProUiElements,
    utils::{overwrite_file, windows_error_dialog},
};
use anyhow::Result;
use egui::{RichText, Ui};
use egui_extras::Column;
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use rand::{rngs::StdRng, seq::SliceRandom};
use std::{collections::HashSet, fmt::Display, fs::File, io::Read, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum PaType {
    #[default]
    None,
    PairedChoice,
}

impl Display for PaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaType::None => write!(f, "None"),
            PaType::PairedChoice => write!(f, "Paired Choice"),
        }
    }
}

// example conditions
// teddy bear
// drum kit
// book
// iPad
// rattle
// ball
// animal toys
// trampoline

pub struct PreferenceAssessment {
    pub conditions_string: String,
    pub conditions: Vec<String>,
    pub all_pairs: Vec<(String, String, bool, bool)>,
    pub ordered: bool,
    pub import_dialog: FileDialog,
    pub export_dialog: FileDialog,
}

impl Default for PreferenceAssessment {
    fn default() -> Self {
        let fd = FileDialog::default().default_file_name("preferences.txt");

        Self {
            conditions_string: Default::default(),
            conditions: Default::default(),
            all_pairs: Default::default(),
            ordered: true,
            import_dialog: FileDialog::default(),
            export_dialog: fd,
        }
    }
}

impl PreferenceAssessment {
    pub fn load_file(&mut self, file_path: PathBuf) -> Result<()> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;

        let mut set = HashSet::new();
        for line in s.lines() {
            if let Some((a, b)) = line.split_once(',') {
                let a = a.trim();
                let b = b.trim();
                self.all_pairs
                    .push((a.to_string(), b.to_string(), false, false));
                set.insert(a.to_string());
                set.insert(b.to_string());
            }
        }
        self.conditions = set.into_iter().collect();
        self.conditions.sort();
        self.conditions_string.clear();
        self.conditions_string = self.conditions.join("\n");

        Ok(())
    }

    pub fn update(&mut self) {
        self.update_conditions();
        self.update_pairs();
    }

    pub fn update_conditions(&mut self) {
        self.conditions = self
            .conditions_string
            .split("\n")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn update_pairs(&mut self) {
        self.all_pairs.clear();
        for (i, a) in self.conditions.iter().enumerate() {
            for (j, b) in self.conditions.iter().enumerate() {
                if !self.ordered {
                    if i > j {
                        continue;
                    }
                }
                if a != b {
                    self.all_pairs.push((a.clone(), b.clone(), false, false));
                }
            }
        }
    }

    pub fn shuffle_pairs(&mut self, rng: &mut StdRng) {
        self.all_pairs.shuffle(rng);
    }

    pub fn import_export(&mut self, ui: &mut Ui) {
        self.import_dialog.update(ui.ctx());
        if let Some(path) = self.import_dialog.take_picked() {
            if let Err(e) = self.load_file(path) {
                windows_error_dialog(e)
            };
        }

        self.export_dialog.update(ui.ctx());
        if let Some(path) = self.export_dialog.take_picked() {
            let data = self
                .all_pairs
                .iter()
                .map(|(a, b, _, _)| format!("{a}, {b}"))
                .join("\n");

            if let Err(e) = overwrite_file(Ok(path), &data) {
                windows_error_dialog(e)
            }
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.preference_assessment.import_export(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Paired Choice");
                        if ui.button("Import").clicked() {
                            app.preference_assessment.import_dialog.pick_file();
                        }
                        if ui.button("Export").clicked() {
                            app.preference_assessment.export_dialog.save_file();
                        }
                    });
                    ui.label("Put each condition on a new line.");
                    if ui
                        .add(
                            egui::TextEdit::multiline(
                                &mut app.preference_assessment.conditions_string,
                            )
                            .hint_text(RichText::from("Condition 1\nCondition 2\nCondition 3")),
                        )
                        .changed()
                    {
                        app.preference_assessment.update();
                    }
                    ui.add_space(5.0);

                    ui.label(format!(
                        "With {} conditions there are {} pairs.",
                        app.preference_assessment.conditions.len(),
                        app.preference_assessment.all_pairs.len()
                    ));
                    ui.add_space(10.0);

                    if ui.button("Shuffle").clicked() {
                        app.preference_assessment.shuffle_pairs(&mut app.rng);
                    }
                    ui.add_space(10.0);

                    ui.return_button(app, |_| {});
                });

                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("paired choice scroller")
                        .min_scrolled_height(550.0)
                        .show(ui, |ui| {
                            egui_extras::TableBuilder::new(ui)
                                .id_salt("frequency")
                                .column(Column::exact(150.0))
                                .column(Column::exact(150.0))
                                .striped(true)
                                .body(|mut body| {
                                    for (a, b, abool, bbool) in
                                        app.preference_assessment.all_pairs.iter_mut()
                                    {
                                        body.row(20.0, |mut row| {
                                            row.col(|ui| {
                                                ui.checkbox(abool, a.as_str());
                                            });
                                            row.col(|ui| {
                                                ui.checkbox(bbool, b.as_str());
                                            });
                                        });
                                    }
                                });
                        });
                })
            });
        });
    }
}
