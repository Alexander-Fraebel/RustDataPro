use egui::{Color32, RichText, Ui};
use std::time::Instant;

/// Need to use a macro to pass around a string literal
macro_rules! timer_format {
    () => {
        "{:.0}:{:05.2}"
    };
}

macro_rules! timer_display {
    ($mins:expr, $secs:expr) => {
        RichText::new(format!(timer_format!(), $mins, $secs)).monospace()
    };
    ($ui:ident, $mins:expr, $secs:expr) => {
        $ui.label(timer_display!($mins, $secs))
    };
    ($ui:ident, $mins:expr, $secs:expr, $color:expr) => {
        $ui.label(timer_display!($mins, $secs).color($color))
    };
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;

#[derive(Default)]
pub struct SimpleTimer {
    time_stamps: Vec<Instant>,
}

impl SimpleTimer {
    /// Pause or unpause.
    pub fn toggle(&mut self) {
        self.time_stamps.push(Instant::now());
    }

    /// If the time is active, pause it. Otherwise do nothing.
    pub fn pause(&mut self) {
        if self.is_active() {
            self.toggle();
        }
    }

    /// Remove the last added time stamp
    pub fn undo(&mut self) -> Option<Instant> {
        self.time_stamps.pop()
    }

    /// Remove all time stamps
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Has the timer been started since it was last reset?
    pub fn was_started(&self) -> bool {
        !self.time_stamps.is_empty()
    }

    /// Is the timer currently active?
    pub fn is_active(&self) -> bool {
        !self.time_stamps.len().is_multiple_of(2)
    }

    /// Is the timer currently paused?
    pub fn is_paused(&self) -> bool {
        self.time_stamps.len().is_multiple_of(2)
    }

    /// How long the timer has been running since it was last started.
    pub fn current_time(&self) -> f32 {
        if self.is_paused() {
            0.0
        } else {
            (Instant::now() - *self.time_stamps.last().unwrap()).as_secs_f32()
        }
    }

    // Current time as minutes and seconds.
    pub fn current_mins_secs(&self) -> (f32, f32) {
        let total = self.current_time();
        ((total / 60.0).trunc(), total % 60.0)
    }

    /// How long the time has been running in total, ignoring time paused.
    pub fn total_time(&self) -> f32 {
        let mut total = 0.0;
        let (chunks, _) = self.time_stamps.as_chunks::<2>();
        for i in chunks {
            total += (i[1] - i[0]).as_secs_f32();
        }
        total += self.current_time();
        total
    }

    // Total time as minutes and seconds.
    pub fn total_mins_secs(&self) -> (f32, f32) {
        let total = self.total_time();
        ((total / 60.0).trunc(), total % 60.0)
    }
}

pub fn view_simple_timer_total(ui: &mut Ui, timer: &SimpleTimer) {
    let (total_mins, total_secs) = timer.total_mins_secs();
    match timer.is_paused() {
        false => {
            ui.request_repaint();
            timer_display!(ui, total_mins, total_secs, ACTIVE_COLOR);
        }

        true => {
            timer_display!(ui, total_mins, total_secs);
        }
    }
}

pub fn view_simple_timer_current(ui: &mut Ui, timer: &SimpleTimer) {
    let (cur_mins, cur_secs) = timer.current_mins_secs();
    match timer.is_paused() {
        false => {
            ui.request_repaint();
            timer_display!(ui, cur_mins, cur_secs, ACTIVE_COLOR);
        }

        true => {
            timer_display!(ui, cur_mins, cur_secs);
        }
    }
}
