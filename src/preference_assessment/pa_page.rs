use crate::{
    app::DataPro,
    quick_error,
    ui_elements::DataProUiElements,
    utils::{overwrite_file, windows_error_dialog},
};
use anyhow::Result;
use egui::{RichText, Ui};
use egui_extras::Column;
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use rand::{rngs::StdRng, seq::SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    fs::File,
    io::Read,
    path::PathBuf,
};

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
    pub conditions: Vec<(String, i32)>,
    pub all_pairs: Vec<(String, String, bool, bool)>,
    pub ordered: bool,
    pub import_dialog: FileDialog,
    pub save_pairs_dialog: FileDialog,
    pub save_results_dialog: FileDialog,
}

impl Default for PreferenceAssessment {
    fn default() -> Self {
        Self {
            conditions_string: Default::default(),
            conditions: Default::default(),
            all_pairs: Default::default(),
            ordered: true,
            import_dialog: FileDialog::default(),
            save_pairs_dialog: FileDialog::default().default_file_name("preference_pairs.txt"),
            save_results_dialog: FileDialog::default().default_file_name("preferences.txt"),
        }
    }
}

impl PreferenceAssessment {
    pub fn load_file(&mut self, file_path: PathBuf) -> Result<()> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;

        self.all_pairs.clear();
        self.conditions_string.clear();

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
        self.conditions = set.into_iter().map(|s| (s, 0)).collect();
        self.conditions.sort();
        self.conditions_string = self.conditions.iter().map(|(s, _)| s).join("\n");

        Ok(())
    }

    pub fn update_counts(&mut self) {
        let mut counts = HashMap::new();
        for condition in self.conditions.iter().map(|(s, _count)| s) {
            counts.insert(condition, 0);
        }
        for (a, b, abool, bbool) in self.all_pairs.iter() {
            if *abool {
                counts.entry(a).and_modify(|e| *e += 1);
            }
            if *bbool {
                counts.entry(b).and_modify(|e| *e += 1);
            }
        }
        self.conditions = counts
            .iter()
            .sorted()
            .map(|(s, c)| (s.to_string(), *c))
            .collect();
    }

    pub fn update_conditions(&mut self) {
        self.conditions = self
            .conditions_string
            .split("\n")
            .map(|s| (s.trim().to_string(), 0))
            .filter(|(s, _count)| !s.is_empty())
            .collect();
        self.all_pairs.clear();
        for (i, (a, _counta)) in self.conditions.iter().enumerate() {
            for (j, (b, _countb)) in self.conditions.iter().enumerate() {
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
            quick_error!(self.load_file(path));
        }

        self.save_pairs_dialog.update(ui.ctx());
        if let Some(path) = self.save_pairs_dialog.take_picked() {
            let data = self
                .all_pairs
                .iter()
                .map(|(a, b, _, _)| format!("{a}, {b}"))
                .join("\n");

            quick_error!(overwrite_file(Ok(path), &data));
        }

        self.save_results_dialog.update(ui.ctx());
        if let Some(path) = self.save_results_dialog.take_picked() {
            let total_picks: f32 = self.conditions.iter().map(|(_, c)| *c as f32).sum();
            let data = self
                .conditions
                .iter()
                .map(|(s, count)| format!("{s}: {:.1}", ((*count as f32) / total_picks) * 100.0))
                .join("\n");
            quick_error!(overwrite_file(Ok(path), &data));
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
                            app.preference_assessment.save_pairs_dialog.save_file();
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
                        app.preference_assessment.update_conditions();
                    }
                    ui.add_space(5.0);

                    ui.label(format!(
                        "With {} conditions there are {} pairs.",
                        app.preference_assessment.conditions.len(),
                        app.preference_assessment.conditions.len()
                            * app.preference_assessment.conditions.len()
                            - app.preference_assessment.conditions.len() // app.preference_assessment.all_pairs.len()
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
                        .min_scrolled_height(600.0)
                        .show(ui, |ui| {
                            egui_extras::TableBuilder::new(ui)
                                .id_salt("frequency")
                                .column(Column::exact(25.0))
                                .column(Column::exact(150.0))
                                .column(Column::exact(150.0))
                                .striped(true)
                                .body(|mut body| {
                                    let mut changes = false;
                                    for (n, (a, b, abool, bbool)) in
                                        app.preference_assessment.all_pairs.iter_mut().enumerate()
                                    {
                                        body.row(20.0, |mut row| {
                                            row.col(|ui| {
                                                ui.monospace(format!("{:>2})", n + 1));
                                            });
                                            row.col(|ui| {
                                                if ui.checkbox(abool, a.as_str()).clicked() {
                                                    *bbool = !*abool;
                                                    changes = true;
                                                }
                                            });
                                            row.col(|ui| {
                                                if ui.checkbox(bbool, b.as_str()).clicked() {
                                                    *abool = !*bbool;
                                                    changes = true;
                                                }
                                            });
                                        });
                                    }
                                    if changes {
                                        app.preference_assessment.update_counts();
                                    }
                                });
                        });
                });

                ui.vertical(|ui| {
                    if ui.button("Save Results").clicked() {
                        app.preference_assessment.save_results_dialog.save_file();
                    }
                    for (item, count) in app.preference_assessment.conditions.iter() {
                        ui.label(format!("{}: {}", item, count));
                    }
                });
            });
        });
    }
}
