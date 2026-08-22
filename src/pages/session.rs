use crate::{
    app::DataPro,
    data::{Data, Ksf, output_data::OutputData, timeline::Timeline},
    display_control::DisplayControl,
    quick_error,
    timer::{Timer, TimerStatus, view_paused_timer, view_stopwatch_ms, view_total_time_ms},
    ui_elements::DataProUiElements,
    utils::{ClickedKeys, date_time_string, overwrite_file, rounded_f32, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use egui::{Color32, Key, Layout, RichText, Ui};
use egui_extras::Column;
use indexmap::IndexMap;

const DESCRIPTION_WIDTH: f32 = 100.0;
const KEY_WIDTH: f32 = 30.0;
const ROW_HEIGHT: f32 = 18.0;
const ROW_FONT_SIZE: f32 = 12.0;
const ACTIVE_COLOR: Color32 = Color32::YELLOW;

const LEFT_MARGIN: f32 = 5.0;
const TOP_MARGIN: f32 = 5.0;

macro_rules! record_keypress {
    ($self:expr, $key:expr, $time:expr) => {
        $self.timeline.push(($key, rounded_f32($time)));
        $self.keypresses_display.push($key.name());
        $self.unpress_available = true;
    };
}

/// Need to use a macro to pass around a string literal
macro_rules! timer_format {
    () => {
        "{:6.1}"
    };
}

macro_rules! active_text {
    ($format:expr, $text:expr) => {
        RichText::new(format!($format, $text))
            .monospace()
            .size(ROW_FONT_SIZE)
            .color(ACTIVE_COLOR)
    };
    ($text:expr) => {
        active_text!("{}", $text)
    };
}

macro_rules! active_cell {
    ($row:ident, $format:expr, $text:expr) => {
        $row.col(|ui| {
            ui.label(active_text!($format, $text));
        });
    };
    ($row:ident, $text:expr) => {
        $row.col(|ui| {
            ui.label(active_text!($text));
        });
    };
}

macro_rules! passive_text {
    ($format:expr, $text:expr) => {
        RichText::new(format!($format, $text))
            .size(ROW_FONT_SIZE)
            .monospace()
    };
    ($text:expr) => {
        passive_text!("{}", $text)
    };
}

macro_rules! passive_cell {
    ($row:ident,$format:expr, $text:expr) => {
        $row.col(|ui| {
            ui.label(passive_text!($format, $text));
        });
    };
    ($row:ident, $text:expr) => {
        $row.col(|ui| {
            ui.label(passive_text!($text));
        });
    };
}

macro_rules! timer_display {
    (active, $row:ident, $desc:ident, $key:ident, $timer:expr, $bouts:expr) => {
        active_cell!($row, $desc);
        active_cell!($row, $key.name());
        active_cell!($row, timer_format!(), $timer.cached.active.saved);
        active_cell!(
            $row,
            timer_format!(),
            $timer.cached.active.last + $timer.current_time()
        );
        active_cell!($row, $bouts);
    };
    (passive, $row:ident, $desc:ident, $key:ident, $timer:expr, $bouts:expr) => {
        passive_cell!($row, $desc);
        passive_cell!($row, $key.name());
        passive_cell!($row, timer_format!(), $timer.cached.active.saved);
        passive_cell!($row, timer_format!(), $timer.cached.active.last);
        passive_cell!($row, $bouts);
    };
}

impl DataPro {
    pub fn save_new_output_data(&mut self) -> Result<()> {
        let file_name = self.path_to_session_records().join(format!(
            "{}-{}_{}{}.txt",
            self.data.active_assessment_name(),
            self.data.active_condition_name(),
            self.data.current_session,
            self.data.session.data_type.abbrev()
        ));
        overwrite_file(Ok(file_name), &self.write_output_json()?)?;
        self.data.increment_current_session();
        self.overwrite_assessments()?;
        Ok(())
    }

    /// Write the output data into a JSON format. Not especially human readable.
    pub fn write_output_json(&self) -> Result<String> {
        let mut fre_map: IndexMap<Key, u32> = IndexMap::new();
        for (t, k, _d) in self.session.freq_keys.iter() {
            fre_map.insert(*k, *t);
        }
        let mut dur_map: IndexMap<Key, (u32, f32)> = IndexMap::new();
        for (t, bouts, k, _d) in self.session.dura_keys.iter() {
            dur_map.insert(*k, (*bouts, rounded_f32(t.active_time())));
        }

        serde_json::to_string(&OutputData {
            datetime: date_time_string(&self.session.start_time),
            session_duration: rounded_f32(self.session.main_timer.active_time()),
            session: self.data.session.clone(),
            duration: dur_map,
            frequency: fre_map,
            timeline: self.session.timeline.clone(),
            ksf: self
                .data
                .ksfs
                .get(self.data.chosen_ksf())
                .unwrap_or(&Ksf::default())
                .clone(),
            client_name: self.data.client.name.clone(),
            client_id: self.data.client.id.clone(),
            case_manager: self.data.client.case_manager.clone(),
            primary_therapist: self.data.client.primary_therapist.clone(),
            session_number: self.data.current_session,
            days_since_admissions: self.data.client.days_since_admission().unwrap_or(i32::MIN), // this should always be valid but avoid crash by giving default
            location: self.data.client.location.clone(),
        })
        .context("failure to create json")
    }
}

pub struct SessionPage {
    pub freq_keys: Vec<(u32, Key, String)>,
    pub dura_keys: Vec<(Timer, u32, Key, String)>,
    pub main_timer: Timer,
    pub pause_timer: Timer,
    pub start_time: DateTime<Local>,
    pub timeline: Timeline,
    pub keypresses_display: Vec<&'static str>,
    pub clicked_keys: ClickedKeys,
    pub save_discard_open: bool,
    pub confirm_end: bool,
    pub unpress_available: bool,
}

impl Default for SessionPage {
    fn default() -> Self {
        Self {
            main_timer: Timer::default(),
            pause_timer: Timer::default(),
            start_time: Local::now(),
            freq_keys: Vec::new(),
            dura_keys: Vec::new(),
            timeline: Timeline::default(),
            keypresses_display: Vec::from(["_"; 11]),
            clicked_keys: ClickedKeys::new(),
            save_discard_open: false,
            confirm_end: false,
            unpress_available: false,
        }
    }
}

impl SessionPage {
    fn reset(&mut self) {
        *self = Self::default()
    }

    /// Stops all timers, records the final keypress (simulates it if session ended another way), disallows unpressing keys, then opens the Save/Discard dialog.
    /// This should only occur once in a session, when it ends.
    fn stop_all_timers(&mut self) {
        if self.main_timer.was_started() {
            for (timer, _, _, _) in self.dura_keys.iter_mut() {
                timer.stop();
            }
            self.main_timer.stop();
            self.timeline
                .push((Key::Escape, rounded_f32(self.main_timer.active_time())));
            self.keypresses_display.push("e");
        }
        self.unpress_available = false;
    }

    /// Pause or unpause all timers, including the session timer. This method MUST be the only way to pause or unpause any timers.
    fn pause_unpause_all_timers(&mut self) {
        for (timer, _, _, _) in self.dura_keys.iter_mut() {
            if timer.was_started() {
                timer.toggle_pause();
            }
        }
        self.main_timer.toggle_pause();
        self.unpress_available = false;
    }

    /// Decrement a key's counter and rewind the recorded time if necessary.
    fn unpress_key(&mut self) {
        if self.unpress_available {
            // Cannot unpress a pause of start of session
            if ["t", "p"].contains(self.keypresses_display.iter().last().unwrap()) {
                self.unpress_available = false;
                return;
            }
            self.keypresses_display.pop();
            if let Some((removed_key, _time)) = self.timeline.pop() {
                for (timer, bouts, key, _) in self.dura_keys.iter_mut() {
                    if key == &removed_key {
                        if timer.is_active() {
                            timer.undo();
                            *bouts = bouts.saturating_sub(1);
                        } else {
                            timer.undo();
                        }
                        if ["t", "p"].contains(self.keypresses_display.iter().last().unwrap()) {
                            self.unpress_available = false;
                            return;
                        }
                        return;
                    }
                }
                for (counter, key, _) in self.freq_keys.iter_mut() {
                    if key == &removed_key {
                        *counter = counter.saturating_sub(1);
                        if ["t", "p"].contains(self.keypresses_display.iter().last().unwrap()) {
                            self.unpress_available = false;
                            return;
                        }
                        return;
                    }
                }
            };
        }
    }

    /// Create the counters and timers defined by the KSF to use in session
    pub fn load_ksf(&mut self, data: &Data) {
        if let Some(active_ksf) = data.ksfs.get(data.chosen_ksf()) {
            let (freq, dura) = active_ksf.pairs();

            for (key, desc) in freq {
                self.freq_keys.push((0, *key, desc.clone()));
            }
            for (key, desc) in dura {
                self.dura_keys
                    .push((Timer::default(), 0, *key, desc.clone()));
            }
        }
    }

    /// Start the session time and record the initial keypress.
    fn start_session(&mut self) {
        self.main_timer.start();
        self.start_time = Local::now();
        self.timeline.push((Key::Tab, 0.0));
        self.keypresses_display.push("t");
    }

    /// Reset the session page and return to the prep session page.
    fn leave_session(&mut self, display_info: &mut DisplayControl) {
        self.reset();
        display_info.go_to_prep_session();
    }
}

impl DataPro {
    pub fn view_session(&mut self, ui: &mut Ui) {
        if self.prep_session.limit_session_length && self.session.main_timer.is_active() {
            if self.session.main_timer.active_time() >= self.prep_session.maximum_session_length {
                self.session.save_discard_open = true;
                self.session.stop_all_timers();
            }
        }

        // Itercept key presses to detect clicks and then delete all of them to prevent egui from reusing them.
        ui.ctx().input_mut(|i| {
            self.session.clicked_keys.update(i);
            i.events.clear();
        });

        // ######################
        // ### Permanent Keys ###
        // ######################
        // Starting is only allowed when session is Stopped.
        if self.session.clicked_keys.contains(&egui::Key::Tab) {
            if !self.session.main_timer.was_started() {
                self.session.start_session();
            }
        }
        // Stop timers and open the confirmation self.session_page.
        if self.session.clicked_keys.contains(&egui::Key::Escape) {
            self.session.confirm_end = true;
        }
        // Pausing can be toggled. Definition of pause prevents this from being used when Stopped.
        if self.session.clicked_keys.contains(&egui::Key::Space) {
            if self.session.main_timer.was_started() {
                self.session.pause_unpause_all_timers();
                self.session.unpress_available = false;
                self.session.keypresses_display.push("p");
            }
        }
        if self.session.clicked_keys.contains(&egui::Key::Backspace) {
            if self.session.main_timer.is_active() {
                self.session.unpress_key();
            }
        }

        // ###################################
        // ### Duration and Frequency Keys ###
        // ###################################
        if self.session.main_timer.is_active() {
            for (timer, bouts, key, _) in self.session.dura_keys.iter_mut() {
                if self.session.clicked_keys.contains(key) {
                    timer.toggle();
                    if timer.is_active() {
                        *bouts += 1;
                    }
                    record_keypress!(self.session, *key, self.session.main_timer.active_time());
                }
            }
            for (counter, key, _) in self.session.freq_keys.iter_mut() {
                if self.session.clicked_keys.contains(key) {
                    *counter += 1;
                    record_keypress!(self.session, *key, self.session.main_timer.active_time());
                }
            }
        }

        // #####################################
        // ### Confirm End of Session Window ###
        // #####################################
        let session_was_started = self.session.main_timer.was_started();
        if self.session.confirm_end {
            egui::Window::new("End Session?").show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].set_height(50.0);
                    if columns[0].large_green_button("YES").clicked() {
                        self.session.confirm_end = false;
                        self.session.save_discard_open = true;
                        self.session.stop_all_timers();
                    }
                    columns[1].set_height(50.0);
                    if columns[1].large_red_button("NO").clicked() {
                        self.session.confirm_end = false;
                    }
                });
            });
        }

        // ##############################
        // ### Save of Discard Window ###
        // ##############################
        if self.session.save_discard_open {
            egui::Window::new("Save Data?").show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].set_height(50.0);
                    columns[0].add_enabled_ui(session_was_started, |ui| {
                        if ui
                            .large_green_button("SAVE")
                            .on_disabled_hover_text("no data to save")
                            .clicked()
                        {
                            quick_error!(self.save_new_output_data());
                            self.session.leave_session(&mut self.display_info);
                        }
                    });
                    columns[1].set_height(50.0);
                    if columns[1].large_red_button("DISCARD").clicked() {
                        self.session.leave_session(&mut self.display_info);
                    }
                });
            });
        }

        // #########################
        // ### Main Display Area ###
        // #########################
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(TOP_MARGIN);
            ui.horizontal(|ui| {
                ui.add_space(LEFT_MARGIN);
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Client ID: {}", self.data.client.id));
                        ui.label(format!("Session Number: {}", self.data.current_session));
                        ui.label(format!(
                            "DOA: {}",
                            self.data.client.days_since_admission().unwrap_or(i32::MIN) //this should always be valid but avoid crash by giving default
                        ));
                        ui.label(format!("Location: {}", self.data.client.location));
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Assessment: {}",
                            self.data.session.chosen_assessment
                        ));
                        ui.label(format!("Condition: {}", self.data.session.chosen_condition));
                        ui.label(format!("KSF: {}", self.data.chosen_ksf()));
                        ui.label(format!("Data Type: {}", self.data.session.data_type));
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Therapist: {}", self.data.session.therapist));
                        ui.label(format!(
                            "Data Collector: {}",
                            self.data.session.data_collector
                        ));

                        ui.label("");
                    });
                });
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(LEFT_MARGIN);
                ui.add_enabled_ui(self.session.main_timer.is_active(), |ui| {
                    ui.spacing_mut().item_spacing = (10.0, 0.0).into();
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Frequency Keys");
                            egui_extras::TableBuilder::new(ui)
                                .id_salt("frequency")
                                .column(Column::exact(DESCRIPTION_WIDTH))
                                .column(Column::exact(KEY_WIDTH))
                                .column(Column::exact(40.0))
                                .striped(true)
                                .min_scrolled_height(500.0)
                                .cell_layout(
                                    Layout::default()
                                        .with_cross_align(egui::Align::RIGHT)
                                        .with_main_align(egui::Align::Center)
                                        .with_main_justify(true),
                                )
                                .body(|mut body| {
                                    body.row(ROW_HEIGHT, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Description");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Key");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Count");
                                        });
                                    });
                                    for (counter, key, desc) in self.session.freq_keys.iter() {
                                        body.row(ROW_HEIGHT, |mut row| {
                                            passive_cell!(row, desc);
                                            passive_cell!(row, key.name());
                                            passive_cell!(row, counter.to_string());
                                        });
                                    }
                                });
                        })
                    });
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Duration Keys");
                            egui_extras::TableBuilder::new(ui)
                                .id_salt("durationkeys")
                                .column(Column::exact(DESCRIPTION_WIDTH))
                                .column(Column::exact(KEY_WIDTH))
                                .column(Column::exact(60.0))
                                .column(Column::exact(60.0))
                                .column(Column::exact(40.0))
                                .striped(true)
                                .min_scrolled_height(500.0)
                                .cell_layout(
                                    Layout::default()
                                        .with_cross_align(egui::Align::RIGHT)
                                        .with_main_align(egui::Align::Center)
                                        .with_main_justify(true),
                                )
                                .body(|mut body| {
                                    body.row(ROW_HEIGHT, |mut row| {
                                        row.col(|ui| {
                                            ui.strong("Description");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Key");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Total");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Current");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Bouts");
                                        });
                                    });
                                    for (timer, bouts, key, desc) in self.session.dura_keys.iter() {
                                        body.row(ROW_HEIGHT, |mut row| {
                                            match timer.current_status() {
                                                TimerStatus::Active => {
                                                    timer_display!(
                                                        active, row, desc, key, timer, bouts
                                                    );
                                                }
                                                TimerStatus::Stopped => {
                                                    timer_display!(
                                                        passive, row, desc, key, timer, bouts
                                                    );
                                                }
                                                TimerStatus::Paused => match timer.cached.status {
                                                    TimerStatus::Active => {
                                                        timer_display!(
                                                            active, row, desc, key, timer, bouts
                                                        );
                                                    }
                                                    _ => {
                                                        timer_display!(
                                                            passive, row, desc, key, timer, bouts
                                                        );
                                                    }
                                                },
                                            };
                                        });
                                    }
                                });
                        })
                    });
                });
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = (0.0, 0.0).into();
                    ui.group(|ui| {
                        ui.heading("Controls");
                        ui.label(
                            "TAB to start.\nESC return to end session.\nSPACE to pause/unpause.",
                        );
                    });
                    ui.horizontal(|ui| {
                        if self.session.main_timer.was_started() {
                            ui.monospace(RichText::new("Session Time:").color(ACTIVE_COLOR));
                        } else {
                            ui.monospace("Session Time:");
                        }
                        view_stopwatch_ms(ui, &mut self.session.main_timer);
                        if self.prep_session.limit_session_length {
                            ui.label(
                                RichText::from(format!(
                                    "  [{:.0}:{:05.2}]",
                                    (self.prep_session.maximum_session_length / 60.0).trunc(),
                                    self.prep_session.maximum_session_length % 60.0
                                ))
                                .strong()
                                .monospace(),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        if self.session.main_timer.was_started() {
                            ui.monospace(RichText::new(" Paused Time:").color(ACTIVE_COLOR));
                        } else {
                            ui.monospace(" Paused Time:");
                        };
                        view_paused_timer(ui, &mut self.session.main_timer);
                    });
                    ui.horizontal(|ui| {
                        if self.session.main_timer.was_started() {
                            ui.monospace(RichText::new("  Total Time:").color(ACTIVE_COLOR));
                        } else {
                            ui.monospace("  Total Time:");
                        };
                        view_total_time_ms(ui, &mut self.session.main_timer);
                    });
                });
            });
            ui.add_space(10.0);

            ui.add_enabled_ui(self.session.main_timer.is_active(), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(LEFT_MARGIN);
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                for k in self.session.keypresses_display
                                    [(self.session.keypresses_display.len() - 10)..]
                                    .iter()
                                {
                                    ui.monospace(*k);
                                }
                            });
                        });
                        if self.session.unpress_available {
                            ui.label("BACKSPACE to undo last entry.");
                        } else {
                            ui.weak("BACKSPACE to undo last entry.")
                                .on_hover_text("Cannot undo pause or start of session.");
                        }
                    })
                });
            });
        });
    }
}
