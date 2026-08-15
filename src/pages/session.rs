use crate::{
    app::DataPro,
    data::{Data, Ksf, output_data::OutputData, timeline::Timeline},
    display_control::DisplayControl,
    quick_error,
    timer::{Timer, view_nonneg_countdown_timer, view_simple_timer},
    ui_elements::DataProUiElements,
    utils::{ClickedKeys, date_time_string, overwrite_file, rounded_f32, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use egui::{Color32, Key, Layout, RichText, Ui};
use egui_extras::Column;
use indexmap::IndexMap;
use std::collections::VecDeque;

const DESCRIPTION_WIDTH: f32 = 100.0;
const KEY_WIDTH: f32 = 30.0;
const ROW_HEIGHT: f32 = 18.0;
const ROW_FONT_SIZE: f32 = 12.0;
const ACTIVE_COLOR: Color32 = Color32::YELLOW;

macro_rules! record_keypress {
    ($self:expr, $key:expr, $time:expr) => {
        $self.timeline.push(($key, rounded_f32($time)));
        $self.keypresses_display.pop_front();
        $self.keypresses_display.push_back($key.name());
        $self.unpress_available = true;
    };
}

/// Need to use a macro to pass around a string literal
macro_rules! timer_format {
    () => {
        "{:7.1}"
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
    (active, $row:ident, $desc:ident, $key:ident, $time1:expr, $time2:expr, $bouts:expr) => {
        // when this is set alter the bg fill when selected
        // $row.set_selected(true);
        active_cell!($row, $desc);
        active_cell!($row, $key.name());
        active_cell!($row, timer_format!(), $time1);
        active_cell!($row, timer_format!(), $time2);
        active_cell!($row, $bouts);
    };
    (passive, $row:ident, $desc:ident, $key:ident, $time1:expr, $time2:expr, $bouts:expr) => {
        passive_cell!($row, $desc);
        passive_cell!($row, $key.name());
        passive_cell!($row, timer_format!(), $time1);
        passive_cell!($row, timer_format!(), $time2);
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
            dur_map.insert(*k, (*bouts, rounded_f32(t.total_time())));
        }

        serde_json::to_string(&OutputData {
            datetime: date_time_string(&self.session.start_time),
            session_duration: rounded_f32(self.session.timer.total_time()),
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
    pub timer: Timer,
    pub start_time: DateTime<Local>,
    pub timeline: Timeline,
    pub keypresses_display: VecDeque<&'static str>,
    pub clicked_keys: ClickedKeys,
    pub save_discard_open: bool,
    pub unpress_available: bool,
}

impl Default for SessionPage {
    fn default() -> Self {
        Self {
            timer: Timer::default(),
            start_time: Local::now(),
            freq_keys: Vec::new(),
            dura_keys: Vec::new(),
            timeline: Timeline::default(),
            keypresses_display: VecDeque::from(["_"; 11]),
            clicked_keys: ClickedKeys::new(),
            save_discard_open: false,
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
        if self.timer.was_started() && !self.timer.is_active() {
            for (timer, _, _, _) in self.dura_keys.iter_mut() {
                timer.pause();
            }
            self.timer.pause();
            self.timeline
                .push((Key::Escape, rounded_f32(self.timer.total_time())));
            self.keypresses_display.pop_front();
            self.keypresses_display.push_back("e");
        }
        self.unpress_available = false;
        self.save_discard_open = true;
    }

    /// Pause or unpause all timers, including the session timer. This method should be the only way to pause or unpause any timers.
    fn pause_unpause_all_timers(&mut self) {
        for (timer, _, _, _) in self.dura_keys.iter_mut() {
            if timer.was_started() {
                timer.toggle();
            }
        }
        self.timer.toggle();
    }

    /// Decrement a key's counter and rewind the recorded time if necessary.
    fn unpress_key(&mut self) {
        if self.unpress_available {
            self.keypresses_display.push_front("_");
            self.keypresses_display.pop_back();
            if let Some((removed_key, _time)) = self.timeline.pop() {
                for (timer, bouts, key, _) in self.dura_keys.iter_mut() {
                    if key == &removed_key {
                        if timer.is_active() {
                            timer.undo();
                            *bouts = bouts.saturating_sub(1);
                        } else {
                            timer.undo();
                        }
                        if *self.keypresses_display.iter().last().unwrap() == "t" {
                            self.unpress_available = false;
                        }
                        return;
                    }
                }
                for (counter, key, _) in self.freq_keys.iter_mut() {
                    if key == &removed_key {
                        *counter = counter.saturating_sub(1);
                        if *self.keypresses_display.iter().last().unwrap() == "t" {
                            self.unpress_available = false;
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
        self.timer.start();
        self.start_time = Local::now();
        self.timeline.push((Key::Tab, 0.0));
        self.keypresses_display.pop_front();
        self.keypresses_display.push_back("t");
    }

    /// Reset the session page and return to the prep session page.
    fn leave_session(&mut self, display_info: &mut DisplayControl) {
        self.reset();
        display_info.go_to_prep_session();
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        if app.prep_session.limit_session_length && app.session.timer.is_active() {
            if app.session.timer.current_time() >= app.prep_session.maximum_session_length {
                app.session.save_discard_open = true;
                app.session.stop_all_timers();
            }
        }

        // Itercept key presses to detect clicks and then delete all of them to prevent egui from reusing them.
        ui.ctx().input_mut(|i| {
            app.session.clicked_keys.update(i);
            i.events.clear();
        });

        // ### Permanent Keys ###
        // Starting is only allowed when session is Stopped.
        if app.session.clicked_keys.contains(&egui::Key::Tab) {
            if !app.session.timer.was_started() {
                app.session.start_session();
            }
        }
        // Stop timers and open the confirmation app.session_page.
        if app.session.clicked_keys.contains(&egui::Key::Escape) {
            app.session.stop_all_timers();
        }
        // Pausing can be toggled. Definition of pause prevents this from being used when Stopped.
        if app.session.clicked_keys.contains(&egui::Key::Space) {
            if app.session.timer.was_started() {
                app.session.pause_unpause_all_timers();
            }
        }
        if app.session.clicked_keys.contains(&egui::Key::Backspace) {
            if app.session.timer.is_active() {
                app.session.unpress_key();
            }
        }

        // ### Duration and Frequency Keys ###
        if app.session.timer.is_active() {
            for (timer, bouts, key, _) in app.session.dura_keys.iter_mut() {
                if app.session.clicked_keys.contains(key) {
                    timer.toggle();
                    if timer.is_active() {
                        *bouts += 1;
                    }
                    record_keypress!(app.session, *key, app.session.timer.total_time());
                }
            }
            for (counter, key, _) in app.session.freq_keys.iter_mut() {
                if app.session.clicked_keys.contains(key) {
                    *counter += 1;
                    record_keypress!(app.session, *key, app.session.timer.total_time());
                }
            }
        }

        let session_was_started = app.session.timer.was_started();
        if app.session.save_discard_open {
            egui::Window::new("Confirm Exit").show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].set_height(50.0);
                    columns[0].add_enabled_ui(session_was_started, |ui| {
                        if ui
                            .large_green_button("SAVE")
                            .on_disabled_hover_text("no data to save")
                            .clicked()
                        {
                            quick_error!(app.save_new_output_data());
                            app.session.leave_session(&mut app.display_info);
                        }
                    });
                    columns[1].set_height(50.0);
                    if columns[1].large_red_button("DISCARD").clicked() {
                        app.session.leave_session(&mut app.display_info);
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            // ui.visuals_mut().selection.bg_fill = Color32::GOLD;
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Client ID: {}", app.data.client.id));
                        ui.label(format!("Session Number: {}", app.data.current_session));
                        ui.label(format!(
                            "DOA: {}",
                            app.data.client.days_since_admission().unwrap_or(i32::MIN) //this should always be valid but avoid crash by giving default
                        ));
                        ui.label(format!("Location: {}", app.data.client.location));
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Assessment: {}",
                            app.data.session.chosen_assessment
                        ));
                        ui.label(format!("Condition: {}", app.data.session.chosen_condition));
                        ui.label(format!("KSF: {}", app.data.chosen_ksf()));
                        ui.label("");
                    });
                });
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Therapist: {}", app.data.session.therapist));
                        ui.label(format!(
                            "Data Collector: {}",
                            app.data.session.data_collector
                        ));
                        ui.label(format!("Data Type: {}", app.data.session.data_type));
                        ui.label("");
                    });
                });
                ui.vertical(|ui| {
                    ui.label("TAB to start.\nESC return to end session.\nSPACE to pause/unpause.");
                    ui.horizontal(|ui| {
                        if app.session.timer.was_started() {
                            ui.label(RichText::new("Session Time:").color(ACTIVE_COLOR));
                        } else {
                            ui.label("Session Time:");
                        }

                        if app.prep_session.limit_session_length {
                            view_nonneg_countdown_timer(ui, &mut app.session.timer);
                            ui.label(
                                RichText::from(format!(
                                    "[{:.0}:{:05.2}]",
                                    (app.prep_session.maximum_session_length / 60.0).trunc(),
                                    app.prep_session.maximum_session_length % 60.0
                                ))
                                .strong()
                                .monospace(),
                            );
                        } else {
                            view_simple_timer(ui, &mut app.session.timer);
                        }
                    });
                });
            });
            ui.add_space(5.0);

            ui.add_enabled_ui(app.session.timer.is_active(), |ui| {
                ui.spacing_mut().item_spacing = (5.0, 0.0).into();
                ui.horizontal(|ui| {
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
                                    for (counter, key, desc) in app.session.freq_keys.iter() {
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
                                            ui.strong("Current");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Total");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Bouts");
                                        });
                                    });
                                    for (timer, bouts, key, desc) in app.session.dura_keys.iter() {
                                        body.row(ROW_HEIGHT, |mut row| {
                                            if !timer.was_started() {
                                                timer_display!(
                                                    passive,
                                                    row,
                                                    desc,
                                                    key,
                                                    timer.cached_time,
                                                    timer.current_time(),
                                                    bouts
                                                );
                                            } else {
                                                timer_display!(
                                                    active,
                                                    row,
                                                    desc,
                                                    key,
                                                    timer.cached_time,
                                                    timer.current_time(),
                                                    bouts
                                                );
                                            }
                                        });
                                    }
                                });
                        })
                    });
                });
            });
            ui.add_space(5.0);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    for k in app.session.keypresses_display.make_contiguous()[1..11].iter() {
                        ui.monospace(*k);
                    }
                });
            });
            ui.label("BACKSPACE to undo last entry.");
        });
    }
}
