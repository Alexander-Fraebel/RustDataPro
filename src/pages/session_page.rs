use crate::{
    app::DataPro,
    configs::{CLIENT_DATA_FILE_NAME, SESSION_DATA_FOLDER_NAME},
    data::{
        Data, Ksf, Timer, TimerStatus, output_data::OutputData, timeline::Timeline,
        view_simple_timer,
    },
    display_control::DisplayControl,
    ui_elements::DataProUiElements,
    utils::{ClickedKeys, date_time_string, rounded_f32, windows_error_dialog},
};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use egui::{Color32, Key, Layout, RichText, Ui};
use egui_extras::Column;
use indexmap::IndexMap;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

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
            .color(Color32::YELLOW)
    };
    ($text:expr) => {
        active_text!("{}", $text)
    };
}

macro_rules! active_row {
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
        RichText::new(format!($format, $text)).monospace()
    };
    ($text:expr) => {
        passive_text!("{}", $text)
    };
}

macro_rules! passive_row {
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
    (bright, $row:ident, $desc:ident, $key:ident, $time1:expr, $time2:expr, $bouts:expr) => {
        active_row!($row, $desc);
        active_row!($row, $key.name());
        active_row!($row, $bouts);
        active_row!($row, timer_format!(), $time1);
        active_row!($row, timer_format!(), $time2);
    };
    (dim, $row:ident, $desc:ident, $key:ident, $time1:expr, $time2:expr, $bouts:expr) => {
        passive_row!($row, $desc);
        passive_row!($row, $key.name());
        passive_row!($row, $bouts);
        passive_row!($row, timer_format!(), $time1);
        passive_row!($row, timer_format!(), $time2);
    };
}

const DESCRIPTION_WIDTH: f32 = 100.0;
const KEY_WIDTH: f32 = 30.0;
const ROW_HEIGHT: f32 = 20.0;

pub struct SessionPage {
    pub freq_keys: Vec<(u32, Key, String)>,
    pub dura_keys: Vec<(Timer, u32, Key, String)>,
    pub session_timer: Timer,
    pub session_start: DateTime<Local>,
    pub timeline: Timeline,
    pub keypresses_display: VecDeque<&'static str>,
    pub clicked_keys: ClickedKeys,
    pub save_discard_open: bool,
    pub unpress_available: bool,
    pub limit_session_length: bool,
    pub maximum_session_length: f32,
}

impl Default for SessionPage {
    fn default() -> Self {
        Self {
            session_timer: Timer::default(),
            session_start: Local::now(),
            freq_keys: Vec::new(),
            dura_keys: Vec::new(),
            timeline: Timeline::default(),
            keypresses_display: VecDeque::from(["_"; 11]),
            clicked_keys: ClickedKeys::new(),
            save_discard_open: false,
            unpress_available: false,
            limit_session_length: false,
            maximum_session_length: 0.0,
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
        if self.session_timer.was_started() && !self.session_timer.is_stopped() {
            for (timer, _, _, _) in self.dura_keys.iter_mut() {
                timer.stop();
            }
            self.session_timer.stop();
            self.timeline
                .push((Key::Escape, rounded_f32(self.session_timer.total_time())));
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
        self.session_timer.toggle();
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
                            timer.unstart();
                            *bouts = bouts.saturating_sub(1);
                        } else {
                            timer.unstop();
                        }
                    }
                }
                for (counter, key, _) in self.freq_keys.iter_mut() {
                    if key == &removed_key {
                        *counter = counter.saturating_sub(1);
                    }
                }
            };
            self.unpress_available = false;
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

    /// Start the session time and ecord the initial keypress.
    fn start_session(&mut self) {
        self.session_timer.start();
        self.session_start = Local::now();
        self.timeline.push((Key::Tab, 0.0));
        self.keypresses_display.pop_front();
        self.keypresses_display.push_back("t");
    }

    /// Reset the session page and return to the prep session page.
    fn leave_session(&mut self, display_info: &mut DisplayControl) {
        self.reset();
        display_info.go_to_prep_session();
    }

    /// Save the output data and increment the session number.
    fn finalize_session(&mut self, data: &mut Data, root_directory: &PathBuf) -> Result<()> {
        self.save_output(data, root_directory)?;
        self.increment_current_session(data, root_directory)?;
        Ok(())
    }

    // /// Write the output data into a human readable format.
    // fn write_output_pretty(&self, data: &Data) -> String {
    //     let mut output = String::new();

    //     output.push_str("---Session---\n");
    //     output.push_str(&data.client.to_string());
    //     output.push('\n');
    //     output.push_str(&format!(
    //         "\nStart {}\nDuration {:.1}\n",
    //         date_time_string(&self.session_start),
    //         self.session_timer.total_time()
    //     ));
    //     output.push('\n');
    //     output.push_str(&data.session.to_string());

    //     output.push_str("\n\n--Duration--\n");

    //     for (timer, bouts, _key, desc) in self.dura_keys.iter() {
    //         output.push_str(&format!(
    //             "{} {:.1} ({} bouts)\n",
    //             desc,
    //             timer.saved_time(),
    //             bouts
    //         ));
    //     }

    //     output.push_str("\n--Frequency--\n");
    //     for (counter, _, desc) in self.freq_keys.iter() {
    //         output.push_str(&format!("{} {}\n", desc, counter));
    //     }

    //     output.push_str("\n--Raw Inputs--\n");
    //     output.push_str(
    //         &self
    //             .timeline
    //             .iter()
    //             .map(|(k, t)| format!("{} {:.1}", k.name(), t))
    //             .join("\n"),
    //     );

    //     output
    // }

    /// Write the output data into a JSON format. Not especially human readable.
    fn write_output_json(&self, data: &Data) -> Result<String> {
        let mut fre_map: IndexMap<Key, u32> = IndexMap::new();
        for (t, k, _d) in self.freq_keys.iter() {
            fre_map.insert(*k, *t);
        }
        let mut dur_map: IndexMap<Key, (u32, f32)> = IndexMap::new();
        for (t, bouts, k, _d) in self.dura_keys.iter() {
            dur_map.insert(*k, (*bouts, rounded_f32(t.total_time())));
        }

        serde_json::to_string(&OutputData {
            datetime: date_time_string(&self.session_start),
            session_duration: rounded_f32(self.session_timer.total_time()),
            session: data.session.clone(),
            duration: dur_map,
            frequency: fre_map,
            timeline: self.timeline.clone(),
            ksf: data
                .ksfs
                .get(data.chosen_ksf())
                .unwrap_or(&Ksf::default())
                .clone(),
            client_name: data.client.name.clone(),
            client_id: data.client.id.clone(),
            case_manager: data.client.case_manager.clone(),
            primary_therapist: data.client.primary_therapist.clone(),
            session_number: data.client.current_session,
            days_since_admissions: data.client.days_since_admission().unwrap_or(i32::MIN), // this should always be valid but avoid crash by giving default
            location: data.client.location.clone(),
        })
        .context("failure to create json")
    }

    /// Write the output to a file.
    fn save_output(&mut self, data: &Data, root_directory: &PathBuf) -> Result<()> {
        let path_to_folder = Path::new(root_directory)
            .join(data.client.id.to_string())
            .join(SESSION_DATA_FOLDER_NAME);

        let mut file_name_raw = path_to_folder.clone();
        file_name_raw.push(format!(
            "{}{}_{}.txt",
            data.client.initials(),
            data.client.current_session,
            data.session.data_type.abbrev()
        ));
        let mut writer = BufWriter::new(File::create(file_name_raw)?);
        writer.write_all(self.write_output_json(data)?.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    /// This assumes that both data collectors use INDEPENDENT files
    fn increment_current_session(
        &mut self,
        data: &mut Data,
        root_directory: &PathBuf,
    ) -> Result<()> {
        let mut path = root_directory.clone();
        path.push(&data.client.id);
        path.push(CLIENT_DATA_FILE_NAME);
        std::fs::write(path, &data.client.to_json()?)?;
        data.client.current_session += 1;
        Ok(())
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        if app.session_page.limit_session_length && app.session_page.session_timer.is_active() {
            if app.session_page.session_timer.current_time()
                >= app.session_page.maximum_session_length
            {
                app.session_page.save_discard_open = true;
                app.session_page.stop_all_timers();
            }
        }

        // Itercept key presses to detect clicks and then delete all of them to prevent egui from reusing them.
        ui.ctx().input_mut(|i| {
            app.session_page.clicked_keys.update(i);
            i.events.clear();
        });

        // ### Permanent Keys ###
        // Starting is only allowed when session is Stopped.
        if app.session_page.clicked_keys.contains(&egui::Key::Tab) {
            if !app.session_page.session_timer.was_started() {
                app.session_page.start_session();
            }
        }
        // Stop timers and open the confirmation app.session_page.
        if app.session_page.clicked_keys.contains(&egui::Key::Escape) {
            app.session_page.stop_all_timers();
        }
        // Pausing can be toggled. Definition of pause prevents this from being used when Stopped.
        if app.session_page.clicked_keys.contains(&egui::Key::Space) {
            if app.session_page.session_timer.was_started() {
                app.session_page.pause_unpause_all_timers();
            }
        }
        if app
            .session_page
            .clicked_keys
            .contains(&egui::Key::Backspace)
        {
            if app.session_page.session_timer.is_active() {
                app.session_page.unpress_key();
            }
        }

        // ### Duration and Frequency Keys ###
        if app.session_page.session_timer.is_active() {
            for (timer, bouts, key, _) in app.session_page.dura_keys.iter_mut() {
                if app.session_page.clicked_keys.contains(key) {
                    timer.stop_start();
                    if timer.is_active() {
                        *bouts += 1;
                    }
                    record_keypress!(
                        app.session_page,
                        *key,
                        app.session_page.session_timer.total_time()
                    );
                }
            }
            for (counter, key, _) in app.session_page.freq_keys.iter_mut() {
                if app.session_page.clicked_keys.contains(key) {
                    *counter += 1;
                    record_keypress!(
                        app.session_page,
                        *key,
                        app.session_page.session_timer.total_time()
                    );
                }
            }
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!("Client ID: {}", app.data.client.id));
                        ui.label(format!(
                            "Session Number: {}",
                            app.data.client.current_session
                        ));
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
            });
            ui.add_space(10.0);

            ui.label("TAB to start. ESC return to end session. SPACE to pause/unpause.");
            match app.session_page.session_timer.status() {
                TimerStatus::Active => {
                    ui.label(RichText::new("ACTIVE").monospace().color(Color32::GREEN))
                }
                TimerStatus::Stopped => {
                    ui.label(RichText::new("STOPPED").monospace().color(Color32::RED))
                }
                TimerStatus::Paused => {
                    ui.label(RichText::new("PAUSED").monospace().color(Color32::YELLOW))
                }
                TimerStatus::NotStarted => {
                    ui.label(RichText::new("NOT STARTED").monospace().color(Color32::RED))
                }
            };
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Session Time:");
                view_simple_timer(ui, &mut app.session_page.session_timer);
                if app.session_page.limit_session_length {
                    ui.monospace(format!(
                        "[{:.0}:{:05.2}]",
                        app.session_page.maximum_session_length / 60.0,
                        app.session_page.maximum_session_length % 60.0
                    ));
                }
            });
            ui.add_space(10.0);
            ui.add_enabled_ui(app.session_page.session_timer.is_active(), |ui| {
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
                                    for (counter, key, desc) in app.session_page.freq_keys.iter() {
                                        body.row(ROW_HEIGHT, |mut row| {
                                            row.col(|ui| {
                                                ui.monospace(desc);
                                            });
                                            row.col(|ui| {
                                                ui.monospace(key.name());
                                            });
                                            row.col(|ui| {
                                                ui.monospace(counter.to_string());
                                            });
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
                                .column(Column::exact(40.0))
                                .column(Column::exact(60.0))
                                .column(Column::exact(60.0))
                                .striped(true)
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
                                            ui.strong("Bouts");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Total");
                                        });
                                        row.col(|ui| {
                                            ui.strong("Current");
                                        });
                                    });
                                    for (timer, bouts, key, desc) in
                                        app.session_page.dura_keys.iter()
                                    {
                                        body.row(ROW_HEIGHT, |mut row| match timer.status() {
                                            TimerStatus::Active | TimerStatus::Paused => {
                                                timer_display!(
                                                    bright,
                                                    row,
                                                    desc,
                                                    key,
                                                    timer.saved_time(),
                                                    timer.current_time(),
                                                    bouts
                                                );
                                            }
                                            TimerStatus::Stopped | TimerStatus::NotStarted => {
                                                timer_display!(
                                                    dim,
                                                    row,
                                                    desc,
                                                    key,
                                                    timer.saved_time(),
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
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    for k in app.session_page.keypresses_display.make_contiguous()[1..11].iter() {
                        ui.monospace(*k);
                    }
                });
            });
            ui.label("BACKSPACE to undo last entry.");

            let session_was_stared = app.session_page.session_timer.was_started();
            if app.session_page.save_discard_open {
                egui::Window::new("Confirm Exit").show(ui, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].set_height(50.0);
                        columns[0].add_enabled_ui(session_was_stared, |ui| {
                            if ui
                                .large_green_button("SAVE")
                                .on_disabled_hover_text("no data to save")
                                .clicked()
                            {
                                if let Err(e) = app
                                    .session_page
                                    .finalize_session(&mut app.data, &app.root_directory)
                                {
                                    windows_error_dialog(e) // application will not leave session until error dialog is closed
                                }
                                app.session_page.leave_session(&mut app.display_info);
                            }
                        });
                        columns[1].set_height(50.0);
                        if columns[1].large_red_button("DISCARD").clicked() {
                            app.session_page.leave_session(&mut app.display_info);
                        }
                    });
                });
            }
        });
    }
}
