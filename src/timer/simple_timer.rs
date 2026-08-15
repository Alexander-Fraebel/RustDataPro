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

fn mins_secs(n: f32) -> (f32, f32) {
    ((n / 60.0).trunc(), n % 60.0)
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;
const NEGATIVE_COLOR: Color32 = Color32::RED;

#[derive(Default)]
pub struct SimpleTimer {
    pub time_stamps: Vec<Instant>,
}

impl SimpleTimer {
    /// Pause or unpause.
    pub fn toggle(&mut self) {
        self.time_stamps.push(Instant::now());
    }

    /// If the time is paused, start it. Otherwise do nothing.
    pub fn start(&mut self) {
        if self.is_paused() {
            self.toggle();
        }
    }

    /// If the timer is active, pause it. Otherwise do nothing.
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

    /// How long the time has been running in total, ignoring time paused.
    pub fn running_time(&self) -> f32 {
        let mut total = 0.0;
        let (chunks, _) = self.time_stamps.as_chunks::<2>();
        for i in chunks {
            total += (i[1] - i[0]).as_secs_f32();
        }
        total += self.current_time();
        total
    }

    /// Time spent paused since the time was first started.
    pub fn paused_time(&self) -> f32 {
        if self.time_stamps.len() < 2 {
            0.0
        } else {
            let mut total = 0.0;
            let (chunks, _) = self.time_stamps[1..].as_chunks::<2>();
            for i in chunks {
                total += (i[1] - i[0]).as_secs_f32();
            }
            total += self.current_time();
            total
        }
    }

    /// Time since the timer was first started including time paused.
    pub fn total_time(&self) -> f32 {
        if self.time_stamps.is_empty() {
            0.0
        } else {
            (self.time_stamps[0] - Instant::now()).as_secs_f32()
        }
    }
}

pub struct Timer {
    pub timer: SimpleTimer,
    pub cached_time: f32,
    pub countdown_from: f32,
    stopped: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            timer: Default::default(),
            cached_time: Default::default(),
            countdown_from: 30.0,
            stopped: false,
        }
    }
}

impl Timer {
    /// Pause or unpause.
    pub fn toggle(&mut self) {
        if !(self.stopped && self.is_paused()) {
            self.timer.toggle();
            self.update_cached_time();
        }
    }

    /// If the timer is paused, start it. Otherwise do nothing.
    pub fn start(&mut self) {
        if self.is_paused() && !self.stopped {
            self.toggle();
        }
    }

    /// If the time is active, pause it. Otherwise do nothing.
    pub fn pause(&mut self) {
        if self.is_active() {
            self.toggle();
        }
    }

    /// Pause the timer and flag it as stopped. When stopped .start() and .toggle() will no longer start the timer.
    pub fn stop(&mut self) {
        self.pause();
        self.stopped = true;
    }

    /// Remove the last added time stamp.
    pub fn undo(&mut self) -> Option<Instant> {
        let out = self.timer.undo();
        self.update_cached_time();
        out
    }

    /// Remove all time stamps and reset cached time.
    pub fn reset(&mut self) {
        *self = Self {
            countdown_from: self.countdown_from,
            ..Default::default()
        }
    }

    /// Has the timer been started since it was last reset?
    pub fn was_started(&self) -> bool {
        self.timer.was_started()
    }

    /// Is the timer currently active?
    pub fn is_active(&self) -> bool {
        self.timer.is_active()
    }

    /// Is the timer currently paused?
    pub fn is_paused(&self) -> bool {
        self.timer.is_paused()
    }

    /// Is the timer currently paused AND flagged as stopped?
    pub fn is_stopped(&self) -> bool {
        self.is_paused() && self.stopped
    }

    /// Update the cached time to be the sum of the previous active periods. This is relatively expensive.
    pub fn update_cached_time(&mut self) {
        self.cached_time = 0.0;
        let (chunks, _) = self.timer.time_stamps.as_chunks::<2>();
        for i in chunks {
            self.cached_time += (i[1] - i[0]).as_secs_f32();
        }
    }

    /// How long the timer has been running since it was last started.
    pub fn current_time(&self) -> f32 {
        self.timer.current_time()
    }

    /// Current time as minutes and seconds.
    pub fn current_mins_secs(&self) -> (f32, f32) {
        mins_secs(self.current_time())
    }

    /// How long the timer has been running in total, ignoring time paused.
    pub fn running_time(&self) -> f32 {
        self.cached_time + self.current_time()
    }

    /// Total time as minutes and seconds.
    pub fn running_mins_secs(&self) -> (f32, f32) {
        mins_secs(self.running_time())
    }

    /// Remaining time in the countdown. May be negative.
    pub fn remaining_time(&self) -> f32 {
        self.countdown_from - self.running_time()
    }

    /// Remaining time as minutes and seconds.
    pub fn remaining_time_mins_secs(&self) -> (f32, f32) {
        mins_secs(self.remaining_time())
    }

    /// Paused time since the timer was started.
    pub fn paused_time(&self) -> f32 {
        self.timer.paused_time()
    }

    /// Paused time as minutes and seconds.
    pub fn paused_mins_secs(&self) -> (f32, f32) {
        mins_secs(self.paused_time())
    }

    /// Total time since the timer was started.
    pub fn total_time(&self) -> f32 {
        self.timer.total_time()
    }

    /// Total time as minutes and seconds.
    pub fn total_mins_secs(&self) -> (f32, f32) {
        mins_secs(self.total_time())
    }
}

pub fn view_simple_timer(ui: &mut Ui, timer: &Timer) {
    let (mins, secs) = mins_secs(timer.running_time());
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
    } else {
        match timer.is_active() {
            true => {
                ui.request_repaint();
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
            false => {
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
        }
    }
}

pub fn view_simple_countdown_timer(ui: &mut Ui, timer: &Timer) {
    let time = timer.remaining_time();
    let (mins, secs) = mins_secs(time.abs());
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
        return;
    }

    if time.is_sign_positive() {
        match timer.is_active() {
            true => {
                ui.request_repaint();
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
            false => {
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
        }
    } else {
        match timer.is_active() {
            true => {
                ui.request_repaint();
                timer_display!(ui, mins, secs, NEGATIVE_COLOR);
            }
            false => {
                timer_display!(ui, mins, secs, NEGATIVE_COLOR);
            }
        }
    }
}

// Special timer for session page which counts down to zero and not below.
pub fn view_nonneg_countdown_timer(ui: &mut Ui, timer: &Timer) {
    let (mins, secs) = mins_secs(timer.remaining_time().max(0.0));
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
    } else {
        match timer.is_active() {
            true => {
                ui.request_repaint();
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
            false => {
                timer_display!(ui, mins, secs, ACTIVE_COLOR);
            }
        }
    }
}
