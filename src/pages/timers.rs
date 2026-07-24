use crate::{
    app::DataPro,
    data::{Timer, view_simple_countdown_timer, view_simple_timer},
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

const NUM_TIMERS: usize = 5;

struct UserTimer {
    timer: Timer,
    linked: bool,
    description: String,
    timer_type: TimerType,
}

impl UserTimer {
    fn new() -> Self {
        Self {
            timer: Timer::default(),
            linked: false,
            description: String::new(),
            timer_type: TimerType::Countdown,
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
        for _ in 0..NUM_TIMERS {
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
    pub fn pause_all_timers(&mut self) {
        for timer in self.timers.iter_mut() {
            if timer.timer.was_started() {
                timer.timer.pause();
            }
        }
    }

    pub fn reset_all_timers(&mut self) {
        for timer in self.timers.iter_mut() {
            timer.timer.reset();
        }
    }

    pub fn view(app: &mut DataPro, ui: &mut Ui) {
        let open = &mut app.display_info.timers_open;
        let mut accept_keyboard_controls = *open;

        egui::Window::new("Timers").open(open).show(ui, |ui| {
            ui.add_space(10.0);

            ui.strong("Controls:");
            ui.label(
                "1-5 to toggle timers.\n0 to toggle linked timers.\nSpace to pause all timers.\nR to reset all timers.",
            );
            ui.add_space(15.0);

            egui::Grid::new("timers_page_grid")
                .striped(true)
                .show(ui, |ui| {
                    for (n, timer) in app.timers.timers.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui
                                .add_sized(
                                    (110.0, 20.0),
                                    egui::TextEdit::singleline(&mut timer.description)
                                        .prefix(format!("{})", n + 1))
                                        .char_limit(11)
                                        .font(TextStyle::Monospace),
                                )
                                .has_focus()
                            {
                                accept_keyboard_controls = false;
                            };
                            ui.add_space(10.0);
                            if timer.timer_type == TimerType::Countdown {
                                view_simple_countdown_timer(ui, &timer.timer);
                            } else {
                                view_simple_timer(ui, &timer.timer);
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
                            if timer.timer_type == TimerType::Countdown {
                                let draginfo = ui.add_sized(counter_adjust_size,
                                    egui::DragValue::new(&mut timer.timer.countdown_from)
                                        .range(0.0..=86400.0),
                                );
                                if draginfo.has_focus() {
                                    accept_keyboard_controls = false;
                                }
                                if draginfo.changed() {
                                    timer.timer.reset();
                                }
                            } else {
                                    ui.add_sized(counter_adjust_size,egui::Label::new("")
                                );
                            }
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
                            ui.add_space(5.0);



                        });

                        ui.end_row();
                    }
                });
            ui.add_space(10.0);
        });
        if accept_keyboard_controls {
            ui.ctx().input_mut(|input| {
                app.timers.clicked_keys.update(input);

                if app.timers.clicked_keys.contains(&Key::Space) {
                    app.timers.pause_all_timers();
                }

                if app.timers.clicked_keys.contains(&Key::R) {
                    app.timers.reset_all_timers();
                }

                // Detect toggle linked
                if app.timers.clicked_keys.contains(&Key::Num0) {
                    for timer in app.timers.timers.iter_mut() {
                        if timer.linked {
                            timer.timer.toggle();
                        }
                    }
                }

                // Detect toggle each
                for (idx, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5]
                    .iter()
                    .enumerate()
                {
                    if app.timers.clicked_keys.contains(key) {
                        app.timers.timers[idx].timer.toggle()
                    }
                }
            });
        }
    }
}
