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

pub fn mins_secs(n: f32) -> (f32, f32) {
    ((n / 60.0).trunc(), n % 60.0)
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;
const NEGATIVE_COLOR: Color32 = Color32::RED;

#[derive(Clone, Copy)]
pub enum TimerStatus {
    Active(Instant),
    Stopped(Instant),
    Paused(Instant),
}

impl TimerStatus {
    pub fn instant(&self) -> &Instant {
        match self {
            Self::Active(instant) => instant,
            Self::Stopped(instant) => instant,
            Self::Paused(instant) => instant,
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Self::Active(_) => true,
            _ => false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self {
            Self::Stopped(_) => true,
            _ => false,
        }
    }

    pub fn is_paused(&self) -> bool {
        match self {
            Self::Paused(_) => true,
            _ => false,
        }
    }
}

// #[derive(Default)]
// pub struct Timestamps(pub Vec<TimerStatus>);

// impl Timestamps {
//     /// Toggle active or paused.
//     pub fn toggle(&mut self) {
//         if self.0.is_empty() {
//             self.start();
//         }
//         if let Some(time) = self.0.last() {
//             match time {
//                 TimerStatus::Active(_) => self.pause(),
//                 TimerStatus::Stopped(_) => self.start(),
//                 TimerStatus::Paused(_) => self.start(),
//             }
//         }
//     }

//     /// Start the timer.
//     pub fn start(&mut self) {
//         self.0.push(TimerStatus::Active(Instant::now()));
//     }

//     /// Pause the timer.
//     pub fn pause(&mut self) {
//         self.0.push(TimerStatus::Paused(Instant::now()));
//     }

//     pub fn stop(&mut self) {
//         self.0.push(TimerStatus::Stopped(Instant::now()));
//     }

//     /// Remove the last added time stamp
//     pub fn undo(&mut self) -> Option<TimerStatus> {
//         self.0.pop()
//     }

//     /// Remove all time stamps
//     pub fn reset(&mut self) {
//         *self = Self::default();
//     }

//     /// Has the timer been started since it was last reset?
//     pub fn was_started(&self) -> bool {
//         !self.0.is_empty()
//     }

//     /// Is the timer currently active?
//     pub fn is_active(&self) -> bool {
//         match self.0.last() {
//             Some(status) => status.is_active(),
//             None => false,
//         }
//     }

//     /// Is the timer currently paused?
//     pub fn is_paused(&self) -> bool {
//         match self.0.last() {
//             Some(status) => status.is_paused(),
//             None => false,
//         }
//     }

//     /// Is the timer currently stopped?
//     pub fn is_stopped(&self) -> bool {
//         match self.0.last() {
//             Some(status) => status.is_stopped(),
//             None => false,
//         }
//     }

//     pub fn last_time(&self) -> f32 {
//         match self.0.last() {
//             Some(timestamp) => (Instant::now() - *timestamp.instant()).as_secs_f32(),
//             None => 0.0,
//         }
//     }

//     pub fn current_active_time(&self) -> f32 {
//         if self.is_active() {
//             self.last_time()
//         } else {
//             0.0
//         }
//     }

//     pub fn current_stopped_time(&self) -> f32 {
//         if self.is_stopped() {
//             self.last_time()
//         } else {
//             0.0
//         }
//     }

//     pub fn current_paused_time(&self) -> f32 {
//         if self.is_paused() {
//             self.last_time()
//         } else {
//             0.0
//         }
//     }

//     /// How long the timer has been running in total in seconds.
//     pub fn active_time(&self) -> f32 {
//         let mut total = 0.0;
//         for window in self.0.windows(2) {
//             if window[0].is_active() {
//                 total += (*window[1].instant() - *window[0].instant()).as_secs_f32()
//             }
//         }
//         total += self.current_active_time();
//         total
//     }

//     /// How long the timer has spent paused since it was first started in seconds.
//     pub fn paused_time(&self) -> f32 {
//         let mut total = 0.0;
//         for window in self.0.windows(2) {
//             if window[0].is_paused() {
//                 total += (*window[1].instant() - *window[0].instant()).as_secs_f32()
//             }
//         }
//         total += self.current_paused_time();
//         total
//     }

//     /// How long the timer has spent paused since it was first started in seconds.
//     pub fn stopped_time(&self) -> f32 {
//         let mut total = 0.0;
//         for window in self.0.windows(2) {
//             if window[0].is_stopped() {
//                 total += (*window[1].instant() - *window[0].instant()).as_secs_f32()
//             }
//         }
//         total += self.current_stopped_time();
//         total
//     }
// }

pub struct Timer {
    pub timestamps: Vec<TimerStatus>,
    pub cached_active_time: f32,
    pub cached_paused_time: f32,
    pub cached_stopped_time: f32,
    pub countdown_from: f32,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            countdown_from: 30.0,
            timestamps: Default::default(),
            cached_active_time: Default::default(),
            cached_paused_time: Default::default(),
            cached_stopped_time: Default::default(),
        }
    }
}

impl Timer {
    /// Start or stop. If the timer has not been started this starts it. Does nothing when the timer is paused.
    pub fn toggle(&mut self) {
        if !self.was_started() {
            self.start();
        } else if self.is_stopped() {
            self.start();
        } else if self.is_active() {
            self.stop();
        }
    }

    /// Pause or unpause. If the timer is stopped it switches to paused. Does nothing if the timer has not been started.
    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.start();
        } else if self.is_active() || self.is_stopped() {
            self.pause();
        }
    }

    /// Push a new Active time to timestamps.
    pub fn start(&mut self) {
        self.timestamps.push(TimerStatus::Active(Instant::now()));
        self.update_cached_times();
    }

    /// Push a new Stopped time to timestamps.
    pub fn stop(&mut self) {
        self.timestamps.push(TimerStatus::Stopped(Instant::now()));
        self.update_cached_times();
    }

    /// Push a new Paused time to timestamps.
    pub fn pause(&mut self) {
        self.timestamps.push(TimerStatus::Paused(Instant::now()));
        self.update_cached_times();
    }

    /// Remove the last added time stamp and return it if it exists.
    pub fn undo(&mut self) -> Option<TimerStatus> {
        let out = self.timestamps.pop();
        self.update_cached_times();
        out
    }

    /// Remove all time stamps and reset cached time.
    pub fn reset(&mut self) {
        *self = Self {
            countdown_from: self.countdown_from,
            ..Default::default()
        }
    }

    /// Update the cached times in order to diplay them properly. This is relatively expensive.
    pub fn update_cached_times(&mut self) {
        self.cached_active_time = 0.0;
        self.cached_stopped_time = 0.0;
        self.cached_paused_time = 0.0;
        for window in self.timestamps.windows(2) {
            let interval_end = *window[1].instant();
            match window[0] {
                TimerStatus::Active(interval_start) => {
                    self.cached_active_time += (interval_end - interval_start).as_secs_f32()
                }
                TimerStatus::Stopped(interval_start) => {
                    self.cached_stopped_time += (interval_end - interval_start).as_secs_f32()
                }
                TimerStatus::Paused(interval_start) => {
                    self.cached_paused_time += (interval_end - interval_start).as_secs_f32()
                }
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
            Some(timestamp) => (Instant::now() - *timestamp.instant()).as_secs_f32(),
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
        self.cached_active_time + self.current_active_time()
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
            (*self.timestamps[0].instant() - Instant::now()).as_secs_f32()
        }
    }

    /// Remaining time in the countdown. May be negative.
    pub fn remaining_time(&self) -> f32 {
        self.countdown_from - self.active_time()
    }
}

pub fn view_simple_timer(ui: &mut Ui, timer: &Timer) {
    let (mins, secs) = mins_secs(timer.active_time());
    if !timer.was_started() {
        timer_display!(ui, mins, secs);
    } else {
        match timer.timestamps.last() {
            Some(status) => match status {
                TimerStatus::Active(_) => {
                    ui.request_repaint();
                    timer_display!(ui, mins, secs, ACTIVE_COLOR)
                }
                TimerStatus::Stopped(_) => timer_display!(ui, mins, secs),
                TimerStatus::Paused(_) => timer_display!(ui, mins, secs),
            },
            None => timer_display!(ui, 0.0, 0.0),
        };
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
