use std::path::PathBuf;

use crate::{
    quick_error,
    utils::{
        ClickedKeys, Timer, overwrite_file, ui_elements::DataProUiElements, view_stopwatch_ms,
    },
};
use anyhow::Result;
use egui::{Key, RichText, Ui};
use egui_file_dialog::FileDialog;

use regex::Regex;
use std::cell::LazyCell;

const TOTAL_SESSION_TIME: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"Total Session Time: ([0-9]+\.[0-9]*) seconds\n"#).unwrap());
const CONDITION_AND_TIME: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"(.+): ([0-9]+\.[0-9]*)\n"#).unwrap());

#[derive(Debug)]
pub struct FreeOperant {
    pub conditions_string: String,
    pub conditions: Vec<(String, Timer)>,
    pub session_timer: Timer,
    pub save_discard_window_open: bool,
    pub clicked_keys: ClickedKeys,
    pub import_dialog: FileDialog,
    pub save_conditions_dialog: FileDialog,
    pub save_results_dialog: FileDialog,
}

impl Default for FreeOperant {
    fn default() -> Self {
        Self {
            conditions_string: Default::default(),
            conditions: Default::default(),
            save_discard_window_open: Default::default(),
            session_timer: Default::default(),
            clicked_keys: Default::default(),
            import_dialog: Default::default(),
            save_conditions_dialog: FileDialog::default()
                .default_file_name("preference_conditions.txt"),
            save_results_dialog: FileDialog::default()
                .default_file_name("free_operant_preferences.txt"),
        }
    }
}

impl FreeOperant {
    pub fn update_conditions(&mut self) {
        self.conditions = self
            .conditions_string
            .split("\n")
            .map(|s| (s.trim().to_string(), Timer::default()))
            .filter(|(s, _count)| !s.is_empty())
            .collect();
    }

    pub fn load_saved_session(&mut self, file_path: PathBuf) -> Result<()> {
        self.conditions.clear();
        self.stop_all_timers();
        self.reset_all_timers();

        let s = std::fs::read_to_string(file_path)?;

        if let Some(caps) = TOTAL_SESSION_TIME.captures(&s) {
            self.session_timer = Timer::with_offset(caps[1].parse::<f32>()?);
        } else {
            return Err(anyhow::anyhow!(
                "Free Operant session files should begin with the text `Total Session Time:`"
            ));
        }

        for caps in CONDITION_AND_TIME.captures_iter(&s) {
            let name = caps[1].to_string();
            let timer = Timer::with_offset(caps[2].parse::<f32>()?);

            self.conditions.push((name, timer));
        }

        Ok(())
    }

    pub fn reset_all_timers(&mut self) {
        self.session_timer.reset();
        for (_, timer) in self.conditions.iter_mut() {
            timer.reset();
        }
    }

    pub fn stop_all_timers(&mut self) {
        self.session_timer.stop();
        for (_, timer) in self.conditions.iter_mut() {
            timer.stop();
        }
    }

    pub fn subdisplay(&mut self, ui: &mut Ui) {
        if self.save_discard_window_open {
            egui::Window::new("Save Data?").show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].set_height(50.0);
                    if columns[0].large_green_button("SAVE").clicked() {
                        self.save_discard_window_open = false;
                        self.save_results_dialog.save_file();
                    }
                    columns[1].set_height(50.0);
                    if columns[1].large_red_button("DISCARD").clicked() {
                        self.save_discard_window_open = false;
                        self.reset_all_timers();
                    }
                });
            });
        }

        self.import_dialog.update(ui.ctx());
        if let Some(path) = self.import_dialog.take_picked() {
            quick_error!(self.load_saved_session(path));
        }

        self.save_results_dialog.update(ui.ctx());
        if let Some(path) = self.save_results_dialog.take_picked() {
            let mut data = format!(
                "Total Session Time: {:.1} seconds\n",
                self.session_timer.active_time()
            );
            self.conditions.iter().for_each(|(s, timer)| {
                data.push_str(&format!("{s}: {:.1}\n", timer.active_time()))
            });
            quick_error!(overwrite_file(Ok(path), &data));
            self.reset_all_timers();
        }

        ui.heading("Free Operant");

        if ui.large_blue_button("Load Session").clicked() {
            self.import_dialog.pick_file();
        }

        ui.label("Put each condition on a new line.");

        ui.add_enabled_ui(self.session_timer.is_stopped(), |ui| {
            if ui.button("update conditions").clicked() {
                self.update_conditions();
            }
            ui.add(
                egui::TextEdit::multiline(&mut self.conditions_string)
                    .hint_text(RichText::from("Condition 1\nCondition 2\nCondition 3")),
            );
        });

        ui.horizontal(|ui| {
            if ui.large_green_button("Begin").clicked() {
                self.session_timer.start();
            }

            if ui.large_red_button("End").clicked() {
                self.stop_all_timers();
                self.save_discard_window_open = true;
            }
        });

        ui.add_enabled_ui(self.session_timer.is_active(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Session Time:");
                view_stopwatch_ms(ui, &self.session_timer);
            });

            for (condition, timer) in self.conditions.iter() {
                ui.horizontal(|ui| {
                    ui.label(condition);
                    view_stopwatch_ms(ui, timer)
                });
            }
        });

        if self.session_timer.is_active() {
            ui.ctx().input_mut(|input| {
                self.clicked_keys.update(input);

                // Detect toggle each
                for (idx, key) in Key::ALL[46..72].iter().enumerate() {
                    if self.clicked_keys.contains_key(key) {
                        if let Some((_, timer)) = self.conditions.get_mut(idx) {
                            timer.toggle();
                        }
                    }
                }
            });
        }
    }
}

impl crate::app::DataPro {
    pub fn view_free_operant(&mut self, ui: &mut Ui) {
        self.preference_assessment.free_operant.subdisplay(ui);
    }
}
