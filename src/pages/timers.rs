use crate::{
    app::DataPro,
    timer::{Timer, view_countdown_hms, view_stopwatch_hms},
    utils::ClickedKeys,
};
use egui::{
    Key::{self},
    TextStyle, Ui,
};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimerType {
    Countdown,
    Stopwatch,
}

impl Display for TimerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerType::Countdown => write!(f, "Countdown"),
            TimerType::Stopwatch => write!(f, "Stopwatch"),
        }
    }
}

struct UserTimer {
    timer: Timer,
    linked: bool,
    description: String,
    timer_type: TimerType,
    countdown_from: f32,
}

impl UserTimer {
    fn new() -> Self {
        Self {
            timer: Timer::default(),
            linked: false,
            description: String::new(),
            timer_type: TimerType::Countdown,
            countdown_from: 30.0,
        }
    }
}

pub struct Timers {
    timers: Vec<UserTimer>,
    clicked_keys: ClickedKeys,
}

impl Default for Timers {
    fn default() -> Self {
        let mut timers = Vec::new();
        for _ in 0..6 {
            timers.push(UserTimer::new());
        }
        timers[0].linked = true;
        timers[1].linked = true;
        Self {
            timers,
            clicked_keys: ClickedKeys::new(),
        }
    }
}

impl Timers {
    pub fn stop_all_timers(&mut self) {
        for timer in self.timers.iter_mut() {
            if timer.timer.was_started() {
                timer.timer.stop();
            }
        }
    }

    pub fn reset_all_timers(&mut self) {
        for timer in self.timers.iter_mut() {
            timer.timer.reset();
        }
    }
}

impl DataPro {
    pub fn view_timers(&mut self, ui: &mut Ui) {
        let mut accept_keyboard_controls = true;

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(10.0);

            ui.strong("Controls:");
            ui.label(
                "1-6 to toggle timers.\n0 to toggle linked timers.\nSpace to pause all timers.\nR to reset all timers.",
            );
            ui.add_space(15.0);

            egui::Grid::new("timers_page_grid")
                .striped(true).min_row_height(25.0)
                .show(ui, |ui| {
                    for (n, timer) in self.timers.timers.iter_mut().enumerate() {
                        ui.horizontal_centered(|ui| {
                            if ui
                                .add_sized(
                                    (125.0, 20.0),
                                    egui::TextEdit::singleline(&mut timer.description)
                                        .prefix(format!("{})", n + 1))
                                        .char_limit(12)
                                        .font(TextStyle::Monospace),
                                )
                                .has_focus()
                            {
                                accept_keyboard_controls = false;
                            };
                            ui.add_space(10.0);

                            match timer.timer_type {
                                TimerType::Countdown => view_countdown_hms(ui, &timer.timer, timer.countdown_from),
                                TimerType::Stopwatch => view_stopwatch_hms(ui, &timer.timer),
                            }
                            ui.add_space(5.0);

                            if ui.button("↺").on_hover_text("reset").clicked() {
                                timer.timer.reset();
                            }
                            ui.add_space(5.0);

                            if timer.linked {
                                ui.checkbox(&mut timer.linked, "").on_hover_text("linked");
                            } else {
                                ui.checkbox(&mut timer.linked, "").on_hover_text("unlinked");
                            }
                            ui.add_space(5.0);

                            let counter_adjust_size = (50.0,20.0);
                            match timer.timer_type {
                                TimerType::Countdown => {
                                    let draginfo = ui.add_sized(counter_adjust_size,
                                        egui::DragValue::new(&mut timer.countdown_from)
                                        .range(0.0..=99999.0),
                                    );
                                    if draginfo.has_focus() {
                                        accept_keyboard_controls = false;
                                    }
                                    if draginfo.changed() {
                                        timer.timer.reset();
                                    }
                                },
                                TimerType::Stopwatch => {
                                    ui.add_sized(counter_adjust_size,egui::Label::new(""));
                                },
                            }                            
                            ui.add_space(5.0);

                            egui::ComboBox::from_id_salt(format!("timer_mode{n}"))
                                .selected_text(timer.timer_type.to_string())
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut timer.timer_type, TimerType::Countdown, "Countdown").clicked() {
                                        timer.timer.reset();
                                    }
                                    if ui.selectable_value(&mut timer.timer_type, TimerType::Stopwatch, "Stopwatch").clicked() {
                                        timer.timer.reset();
                                    }
                                });
                        });
                        ui.end_row();
                    }
                });
            ui.add_space(10.0);
        });
        if accept_keyboard_controls {
            ui.ctx().input_mut(|input| {
                self.timers.clicked_keys.update(input);

                if self.timers.clicked_keys.contains_key(&Key::Space) {
                    self.timers.stop_all_timers();
                }

                if self.timers.clicked_keys.contains_key(&Key::R) {
                    self.timers.reset_all_timers();
                }

                // Detect toggle linked
                if self.timers.clicked_keys.contains_key(&Key::Num0) {
                    for timer in self.timers.timers.iter_mut() {
                        if timer.linked {
                            timer.timer.toggle();
                        }
                    }
                }

                // Detect toggle each
                for (idx, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6]
                    .iter()
                    .enumerate()
                {
                    if self.timers.clicked_keys.contains_key(key) {
                        self.timers.timers[idx].timer.toggle()
                    }
                }
            });
        }
    }
}
