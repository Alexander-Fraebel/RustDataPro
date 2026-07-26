use egui::{Color32, RichText, Ui};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, time::Instant};

/// Need to use a macro to pass around a string literal
macro_rules! timer_format {
    () => {
        "{:4.0}:{:05.2}"
    };
}

macro_rules! timer_display {
    ($timer:expr) => {
        RichText::new(format!(timer_format!(), $timer / 60.0, $timer % 60.0)).monospace()
    };
    ($ui:ident, $timer:expr) => {
        $ui.label(timer_display!($timer))
    };
    ($ui:ident, $timer:expr, $color:expr) => {
        $ui.label(timer_display!($timer).color($color))
    };
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;
const NEGATIVE_COLOR: Color32 = Color32::RED;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerStatus {
    #[default]
    NotStarted,
    Active,
    Stopped,
    Paused,
}

impl Display for TimerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerStatus::NotStarted => write!(f, "NotStarted"),
            TimerStatus::Active => write!(f, "Active"),
            TimerStatus::Stopped => write!(f, "Stopped"),
            TimerStatus::Paused => write!(f, "Paused"),
        }
    }
}

impl TimerStatus {
    pub fn was_started(&self) -> bool {
        *self != TimerStatus::NotStarted
    }

    pub fn is_active(&self) -> bool {
        *self == TimerStatus::Active
    }

    pub fn is_paused(&self) -> bool {
        *self == TimerStatus::Paused
    }

    pub fn is_stopped(&self) -> bool {
        *self == TimerStatus::Stopped
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    start_time: Instant,
    saved_time: f32,
    stashed_time: f32,
    pub countdown_from: f32,
    status: TimerStatus,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            saved_time: 0.0,
            stashed_time: 0.0,
            countdown_from: 30.0,
            status: TimerStatus::NotStarted,
        }
    }
}

impl Timer {
    /// Pause or unpause. Preferred interface for a Timer.
    pub fn toggle(&mut self) {
        match self.status {
            TimerStatus::Active => self.pause(),
            TimerStatus::Paused | TimerStatus::NotStarted => self.start(),
            TimerStatus::Stopped => (),
        }
    }

    /// Set status to Active and update the most recently started time.
    /// Does nothing if status is Active.
    pub fn start(&mut self) {
        if !self.status.is_active() {
            self.status = TimerStatus::Active;
            self.start_time = Instant::now();
        }
    }

    /// If status is Active, set status to Paused and update the stashed time.
    pub fn pause(&mut self) {
        if self.is_active() {
            self.status = TimerStatus::Paused;
            self.stashed_time += self.elapsed_time();
        }
    }

    /// If Active update the saved time with the elapsed time and the stashed time. If Paused update the saved time with only the stashed time.
    /// The start time is saved in order to inform .unstop() of how long the time was stopped
    /// Then sets the status to Stopped and clears the stashed time.
    /// Prefer .pause() for typical timer behavior. This method is used for finalization on the session page.
    pub fn stop(&mut self) {
        if self.is_active() {
            self.stashed_time += self.elapsed_time();
            self.saved_time += self.stashed_time;
            self.stashed_time = 0.0;
            self.start_time = Instant::now();
            self.status = TimerStatus::Stopped;
        }
        if self.is_paused() {
            self.saved_time += self.stashed_time;
            self.status = TimerStatus::Stopped;
            self.start_time = Instant::now();
            self.stashed_time = 0.0;
        }
    }

    /// For undoing a key press on a stopped timer on the session page.
    pub fn unstart(&mut self) {
        if self.is_active() {
            self.status = TimerStatus::Stopped;
            self.stashed_time = 0.0;
        }
    }

    /// For undoing a key press on an active timer on the session page.
    pub fn unstop(&mut self) {
        if self.is_stopped() {
            self.status = TimerStatus::Active;
            let t = self.elapsed_time();
            self.saved_time -= t;
            self.stashed_time += t;
        }
    }

    /// Stop or start. Does nothing if the timer is Paused.
    /// Specific for use on the session page to update saved time and clear stashed time.
    pub fn stop_start(&mut self) {
        match self.status {
            TimerStatus::Active => self.stop(),
            TimerStatus::Stopped | TimerStatus::NotStarted => self.start(),
            TimerStatus::Paused => (),
        }
    }

    /// Reset all values except countdown_from.
    pub fn reset(&mut self) {
        *self = Self {
            countdown_from: self.countdown_from,
            ..Default::default()
        };
    }

    pub fn status(&self) -> TimerStatus {
        self.status
    }

    /// Is the timer currently in the Active state.
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Is the timer currently in the Paused state.
    pub fn is_paused(&self) -> bool {
        self.status.is_paused()
    }

    /// Is the timer currently in the Stopped state.
    pub fn is_stopped(&self) -> bool {
        self.status.is_stopped()
    }

    /// Has the timer been started at least once since it was last reset?
    pub fn was_started(&self) -> bool {
        self.status.was_started()
    }

    /// Time since the timer was last started in seconds.
    pub fn elapsed_time(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    /// The amount of time currently saved in seconds.
    pub fn saved_time(&self) -> f32 {
        self.saved_time
    }

    /// Time stashed for a short period. Used for .unstop() and for .current_time() calculations.
    pub fn stashed_time(&self) -> f32 {
        self.stashed_time
    }

    /// How long the timer has been running since it was last started, ignoring time paused.
    pub fn current_time(&self) -> f32 {
        match self.status {
            TimerStatus::Active => self.elapsed_time() + self.stashed_time(),
            TimerStatus::Stopped => self.stashed_time(),
            TimerStatus::Paused => self.stashed_time(),
            TimerStatus::NotStarted => 0.0,
        }
    }

    /// The total time recorded in seconds, ignoring time paused. Sum of .saved_time() and .current_time().
    pub fn total_time(&self) -> f32 {
        self.saved_time() + self.current_time()
    }

    /// Time remaining in the countdown. May be negative.
    pub fn remaining_time(&self) -> f32 {
        self.countdown_from - self.total_time()
    }
}

pub fn view_simple_timer(ui: &mut Ui, timer: &Timer) {
    match timer.status {
        TimerStatus::Active => {
            ui.request_repaint();
            timer_display!(ui, timer.total_time(), ACTIVE_COLOR);
        }
        TimerStatus::Stopped => {
            timer_display!(ui, timer.saved_time());
        }
        TimerStatus::Paused => {
            timer_display!(ui, timer.stashed_time(), ACTIVE_COLOR);
        }
        TimerStatus::NotStarted => {
            timer_display!(ui, 0.0);
        }
    }
}

pub fn view_simple_countdown_timer(ui: &mut Ui, timer: &Timer) {
    match timer.status {
        TimerStatus::Active => {
            ui.request_repaint();
            let t = timer.remaining_time();
            if t.is_sign_positive() {
                timer_display!(ui, t, ACTIVE_COLOR);
            } else {
                timer_display!(ui, -t, NEGATIVE_COLOR);
            }
        }
        // Currently Stopped is not possible for a countdown timer via any interface
        // Unsure if this is correct
        TimerStatus::Stopped => {
            let t = timer.countdown_from - timer.saved_time();
            if t.is_sign_positive() {
                timer_display!(ui, t, ACTIVE_COLOR);
            } else {
                timer_display!(ui, -t, NEGATIVE_COLOR);
            }
        }
        TimerStatus::Paused => {
            let t = timer.countdown_from - timer.stashed_time();
            if t.is_sign_positive() {
                timer_display!(ui, t, ACTIVE_COLOR);
            } else {
                timer_display!(ui, -t, NEGATIVE_COLOR);
            }
        }
        TimerStatus::NotStarted => {
            timer_display!(ui, timer.countdown_from);
        }
    }
}
