use egui::{Color32, RichText, Ui};
use std::time::Instant;

/// Need to use a macro to pass around a string literal
macro_rules! timer_format {
    () => {
        "{:4.0}:{:05.2}"
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

pub fn mins_secs(n: f32) -> (f32, f32) {
    ((n / 60.0).trunc(), n % 60.0)
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;
const NEGATIVE_COLOR: Color32 = Color32::RED;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimerStatus {
    Active,
    Stopped,
    Paused,
}

impl TimerStatus {
    pub fn is_active(&self) -> bool {
        *self == Self::Active
    }

    pub fn is_stopped(&self) -> bool {
        *self == Self::Stopped
    }

    pub fn is_paused(&self) -> bool {
        *self == Self::Paused
    }
}

pub struct Timestamp {
    pub status: TimerStatus,
    pub instant: Instant,
}

impl Timestamp {
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn is_stopped(&self) -> bool {
        self.status.is_stopped()
    }

    pub fn is_paused(&self) -> bool {
        self.status.is_paused()
    }

    pub fn active() -> Self {
        Self {
            status: TimerStatus::Active,
            instant: Instant::now(),
        }
    }

    pub fn stopped() -> Self {
        Self {
            status: TimerStatus::Stopped,
            instant: Instant::now(),
        }
    }

    pub fn paused() -> Self {
        Self {
            status: TimerStatus::Paused,
            instant: Instant::now(),
        }
    }
}

pub struct Timer {
    pub timestamps: Vec<Timestamp>,
    pub cached_active_time: f32,
    pub cached_paused_time: f32,
    pub cached_stopped_time: f32,
    pub last_active_time: f32,
    pub countdown_from: f32,
    pub last_status: TimerStatus,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            timestamps: Default::default(),
            cached_active_time: Default::default(),
            cached_paused_time: Default::default(),
            cached_stopped_time: Default::default(),
            last_active_time: Default::default(),
            countdown_from: 30.0,
            last_status: TimerStatus::Stopped,
        }
    }
}

impl Timer {
    /// Start or stop. Preferred interface for the timer.
    /// Updates cached times automatically. If the timer has not been started this starts it. Does nothing when the timer is paused.
    pub fn toggle(&mut self) {
        self.last_active_time = 0.0;
        if !self.was_started() {
            self.start();
        } else if self.is_stopped() {
            self.start();
        } else if self.is_active() {
            self.stop();
        }
        self.update_cached_times();
    }

    /// Pause or unpause. Preferred interface to pause the timer.
    /// Saves and restores the previous status. Does not update cached times.
    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.unpause();
        } else {
            self.pause();
        }
    }

    /// Push a new Active time to timestamps.
    pub fn start(&mut self) {
        self.timestamps.push(Timestamp::active());
    }

    /// Push a new Stopped time to timestamps.
    pub fn stop(&mut self) {
        self.timestamps.push(Timestamp::stopped());
    }

    /// Push a new Paused time to timestamps. Saves the current timer status.
    pub fn pause(&mut self) {
        self.last_active_time += self.current_active_time();
        self.last_status = self
            .timestamps
            .iter()
            .map(|t| t.status)
            .last()
            .unwrap_or(TimerStatus::Stopped);
        self.timestamps.push(Timestamp::paused());
    }

    pub fn unpause(&mut self) {
        if self.is_paused() {
            match self.last_status {
                TimerStatus::Active => self.start(),
                TimerStatus::Stopped => self.stop(),
                TimerStatus::Paused => self.pause(),
            }
        }
    }

    /// Remove the last added time stamp and return it if it exists.
    pub fn undo(&mut self) -> Option<Timestamp> {
        let out = self.timestamps.pop();
        self.update_cached_times();
        out
    }

    /// Remove all time stamps and reset cached times.
    pub fn reset(&mut self) {
        *self = Self {
            countdown_from: self.countdown_from,
            ..Default::default()
        }
    }

    /// Update the cached times for active, stopped, and paused. This is relatively expensive and should only be called when new inputs are made.
    pub fn update_cached_times(&mut self) {
        self.cached_active_time = 0.0;
        self.cached_stopped_time = 0.0;
        self.cached_paused_time = 0.0;
        for window in self.timestamps.windows(2) {
            let interval_end = window[1].instant;
            let interval_start = window[0].instant;
            let interval_length = (interval_end - interval_start).as_secs_f32();
            match window[0].status {
                TimerStatus::Active => self.cached_active_time += interval_length,
                TimerStatus::Stopped => self.cached_stopped_time += interval_length,
                TimerStatus::Paused => self.cached_paused_time += interval_length,
            }
        }
    }

    /// Has the timer been started since it was last reset?
    pub fn was_started(&self) -> bool {
        !self.timestamps.is_empty()
    }

    /// Is the timer currently active?
    pub fn is_active(&self) -> bool {
        match self.timestamps.last() {
            Some(status) => status.is_active(),
            None => false,
        }
    }

    /// Is the timer currently paused?
    pub fn is_paused(&self) -> bool {
        match self.timestamps.last() {
            Some(status) => status.is_paused(),
            None => false,
        }
    }

    /// Is the timer currently stopped?
    pub fn is_stopped(&self) -> bool {
        match self.timestamps.last() {
            Some(status) => status.is_stopped(),
            None => false,
        }
    }

    pub fn last_time(&self) -> f32 {
        match self.timestamps.last() {
            Some(timestamp) => (Instant::now() - timestamp.instant).as_secs_f32(),
            None => 0.0,
        }
    }

    pub fn current_active_time(&self) -> f32 {
        if self.is_active() {
            self.last_time()
        } else {
            0.0
        }
    }

    pub fn current_stopped_time(&self) -> f32 {
        if self.is_stopped() {
            self.last_time()
        } else {
            0.0
        }
    }

    pub fn current_paused_time(&self) -> f32 {
        if self.is_paused() {
            self.last_time()
        } else {
            0.0
        }
    }

    /// How long the timer has been active in seconds.
    pub fn active_time(&self) -> f32 {
        self.last_active_time + self.cached_active_time + self.current_active_time()
    }

    /// How long the timer has been paused in seconds.
    pub fn paused_time(&self) -> f32 {
        self.cached_paused_time + self.current_paused_time()
    }

    /// How long the timer has been stopped in seconds.
    pub fn stopped_time(&self) -> f32 {
        self.cached_stopped_time + self.current_stopped_time()
    }

    /// Duration since the first time stamp in seconds.
    pub fn total_time(&self) -> f32 {
        if self.timestamps.is_empty() {
            0.0
        } else {
            (self.timestamps[0].instant - Instant::now()).as_secs_f32()
        }
    }

    /// Remaining time in the countdown. May be negative.
    pub fn remaining_time(&self) -> f32 {
        self.countdown_from - self.active_time()
    }

    /// Most recent status added to timestamps. Returns Stopped if timestamps is empty.
    pub fn current_status(&self) -> TimerStatus {
        self.timestamps
            .iter()
            .map(|t| t.status)
            .last()
            .unwrap_or(TimerStatus::Stopped)
    }
}

pub fn view_simple_timer(ui: &mut Ui, timer: &Timer) {
    let (mins, secs) = mins_secs(timer.active_time());
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
    } else {
        timer_display!(ui, mins, secs, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint()
        }
    }
}

pub fn view_simple_countdown_timer(ui: &mut Ui, timer: &Timer) {
    let time = timer.remaining_time();
    let (mins, secs) = mins_secs(time);
    if !timer.was_started() {
        timer_display!(ui, mins, secs.abs());
        return;
    }

    if time.is_sign_positive() {
        timer_display!(ui, mins, secs.abs(), ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint();
        }
    } else {
        timer_display!(ui, mins, secs.abs(), NEGATIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint();
        }
    }
}

// Special timer for session page which counts down to zero and not below.
pub fn view_nonneg_countdown_timer(ui: &mut Ui, timer: &Timer) {
    let (mins, secs) = mins_secs(timer.remaining_time().max(0.0));
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
    } else {
        timer_display!(ui, mins, secs, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint();
        }
    }
}
