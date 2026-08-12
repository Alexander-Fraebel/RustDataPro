use crate::{
    app::DataPro,
    ui_elements::DataProUiElements,
    utils::{overwrite_file, windows_error_dialog},
};
use egui::{RichText, Ui};
use egui_extras::Column;
use egui_file_dialog::FileDialog;
use itertools::Itertools;
use rand::{rngs::StdRng, seq::SliceRandom};
use std::fmt::Display;

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
    pub all_pairs: Vec<(String, String)>,
    pub ordered: bool,
    pub file_dialog: FileDialog,
}

impl Default for PreferenceAssessment {
    fn default() -> Self {
        let fd = FileDialog::default().default_file_name("preferences.txt");

        Self {
            conditions_string: Default::default(),
            conditions: Default::default(),
            all_pairs: Default::default(),
            ordered: true,
            file_dialog: fd,
        }
    }
}

impl PreferenceAssessment {
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
                    self.all_pairs.push((a.clone(), b.clone()));
                }
            }
        }
    }

    pub fn shuffle_pairs(&mut self, rng: &mut StdRng) {
        self.all_pairs.shuffle(rng);
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        app.preference_assessment.file_dialog.update(ui.ctx());
        if let Some(path) = app.preference_assessment.file_dialog.take_picked() {
            let data = app
                .preference_assessment
                .all_pairs
                .iter()
                .map(|(a, b)| format!("{a}, {b}"))
                .join("\n");

            if let Err(e) = overwrite_file(Ok(path), &data) {
                windows_error_dialog(e)
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Paired Choice");
                        if ui.button("Shuffle").clicked() {
                            app.preference_assessment.shuffle_pairs(&mut app.rng);
                        }
                        if ui.button("Export").clicked() {
                            app.preference_assessment.file_dialog.save_file();
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

                    ui.return_button(app, |_| {});
                });

                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("paired choice scroller")
                        .min_scrolled_height(550.0)
                        .show(ui, |ui| {
                            egui_extras::TableBuilder::new(ui)
                                .id_salt("frequency")
                                .column(Column::exact(125.0))
                                .column(Column::exact(125.0))
                                .striped(true)
                                .body(|mut body| {
                                    for (a, b) in app.preference_assessment.all_pairs.iter() {
                                        body.row(20.0, |mut row| {
                                            row.col(|ui| {
                                                ui.label(a);
                                            });
                                            row.col(|ui| {
                                                ui.label(b);
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
