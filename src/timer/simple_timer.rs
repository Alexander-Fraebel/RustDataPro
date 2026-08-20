use egui::{Color32, RichText, Ui};
use std::time::Instant;

/// Time display with minutes:seconds.hundredths
/// Allocates space for 10 symbols in total.
/// Max value before additional space is used is 9999:59.99 which is about 7 days
macro_rules! timer_display_ms {
    ($time:expr) => {
        RichText::new(format!(
            "{:4.0}:{:05.2}",
            ($time / 60.0).trunc(), // minutes, maybe negative
            ($time % 60.0).abs()    // seconds, always positive
        ))
        .monospace()
    };
    ($ui:ident, $time:expr) => {
        $ui.label(timer_display_ms!($time))
    };
    ($ui:ident, $time:expr, $color:expr) => {
        $ui.label(timer_display_ms!($time).color($color))
    };
}

// Timer display with hours:minutes:seconds.hundredths
// Allocates space for 11 symbols in total.
// Max value before additional space is used is 9:59:59.99 which is about 10 hours
macro_rules! timer_display_hms {
    ($time:expr) => {
        RichText::new(format!(
            "{:2.0}:{:02.0}:{:05.2}",
            ($time / 3600.0).trunc(),            // hours, maybe negative
            ($time.abs() / 60.0).trunc() % 60.0, // minutes, always positive
            $time.abs() % 60.0,                  // seconds, always positive
        ))
        .monospace()
    };
    ($ui:ident, $time:expr) => {
        $ui.label(timer_display_hms!($time))
    };
    ($ui:ident, $time:expr, $color:expr) => {
        $ui.label(timer_display_hms!($time).color($color))
    };
}

const ACTIVE_COLOR: Color32 = Color32::YELLOW;
const NEGATIVE_COLOR: Color32 = Color32::RED;

#[derive(Clone, Copy, Default)]
pub struct CachedTime {
    pub saved: f32,
    pub last: f32,
}

#[derive(Clone, Copy, Default)]
pub struct CachedInfo {
    pub active: CachedTime,
    pub stopped: CachedTime,
    pub paused: CachedTime,
    pub status: TimerStatus,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerStatus {
    Active,
    #[default]
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
    pub cached: CachedInfo,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            timestamps: Default::default(),
            cached: CachedInfo::default(),
        }
    }
}

impl Timer {
    /// Start or stop. Preferred interface for the timer.
    /// Updates cached times automatically.
    /// If the timer has not been started this starts it. Does nothing when the timer is paused.
    pub fn toggle(&mut self) {
        if !self.was_started() {
            self.start();
        } else if self.is_stopped() {
            self.start();
        } else if self.is_active() {
            self.stop();
        }
    }

    /// Pause or unpause. Used to pause the timer AND override the normal toggle.
    /// Saves the previous status when pausing and restores it when unpausing.
    pub fn toggle_pause(&mut self) {
        if self.is_paused() {
            self.unpause();
        } else {
            self.pause();
        }
    }

    /// Push a new Active time to timestamps.
    pub fn start(&mut self) {
        self.reset_last_active_time();
        self.timestamps.push(Timestamp::active());
        self.update_saved_times();
    }

    /// Push a new Active time to timestamps without updating cached times.
    pub fn start_silent(&mut self) {
        self.timestamps.push(Timestamp::active());
    }

    /// Push a new Stopped time to timestamps.
    pub fn stop(&mut self) {
        self.reset_last_active_time();
        self.timestamps.push(Timestamp::stopped());
        self.update_saved_times();
    }

    /// Push a new Stopped time to timestamps without updating cached times.
    pub fn stop_silent(&mut self) {
        self.timestamps.push(Timestamp::stopped());
    }

    /// Push a new Paused time to timestamps. Saves the current timer status and updates the last active time.
    /// Does not update any cached times.
    pub fn pause(&mut self) {
        self.update_last_active_time();
        self.cached.status = self.current_status();
        self.timestamps.push(Timestamp::paused());
    }

    /// Push a new timestamp of the same type as the last status. Updates the paused time.
    /// Does not update other cached times.
    pub fn unpause(&mut self) {
        match self.cached.status {
            TimerStatus::Active => self.start_silent(),
            TimerStatus::Stopped => self.stop_silent(),
            TimerStatus::Paused => (),
        }
        self.update_paused_time();
    }

    /// Remove the last added time stamp and return it if it exists.
    pub fn undo(&mut self) -> Option<Timestamp> {
        let out = self.timestamps.pop();
        self.update_saved_times();
        out
    }

    /// Remove all time stamps and reset all cached information.
    pub fn reset(&mut self) {
        *self = Self {
            ..Default::default()
        }
    }

    /// Recalculate the saved times for active, stopped, and paused. This is relatively expensive and should only be called when new inputs are made.
    pub fn update_saved_times(&mut self) {
        self.cached.active.saved = 0.0;
        self.cached.stopped.saved = 0.0;
        self.cached.paused.saved = 0.0;
        for window in self.timestamps.windows(2) {
            let interval_end = window[1].instant;
            let interval_start = window[0].instant;
            let interval_length = (interval_end - interval_start).as_secs_f32();
            match window[0].status {
                TimerStatus::Active => self.cached.active.saved += interval_length,
                TimerStatus::Stopped => self.cached.stopped.saved += interval_length,
                TimerStatus::Paused => self.cached.paused.saved += interval_length,
            }
        }
    }

    pub fn update_active_time(&mut self) {
        self.cached.active.saved = 0.0;
        for window in self.timestamps.windows(2) {
            match window[0].status {
                TimerStatus::Active => {
                    let interval_end = window[1].instant;
                    let interval_start = window[0].instant;
                    let interval_length = (interval_end - interval_start).as_secs_f32();
                    self.cached.active.saved += interval_length
                }
                _ => (),
            }
        }
    }

    pub fn update_last_active_time(&mut self) {
        self.cached.active.last += self.current_active_time();
    }

    pub fn reset_last_active_time(&mut self) {
        self.cached.active.last = 0.0;
    }

    pub fn update_stopped_time(&mut self) {
        self.cached.stopped.saved = 0.0;
        for window in self.timestamps.windows(2) {
            match window[0].status {
                TimerStatus::Stopped => {
                    let interval_end = window[1].instant;
                    let interval_start = window[0].instant;
                    let interval_length = (interval_end - interval_start).as_secs_f32();
                    self.cached.stopped.saved += interval_length
                }
                _ => (),
            }
        }
    }

    pub fn update_last_stopped_time(&mut self) {
        self.cached.stopped.last += self.current_stopped_time();
    }

    pub fn reset_last_stopped_time(&mut self) {
        self.cached.stopped.last = 0.0;
    }

    pub fn update_paused_time(&mut self) {
        self.cached.paused.saved = 0.0;
        for window in self.timestamps.windows(2) {
            match window[0].status {
                TimerStatus::Paused => {
                    let interval_end = window[1].instant;
                    let interval_start = window[0].instant;
                    let interval_length = (interval_end - interval_start).as_secs_f32();
                    self.cached.paused.saved += interval_length
                }
                _ => (),
            }
        }
    }

    pub fn update_last_paused_time(&mut self) {
        self.cached.paused.last += self.current_paused_time();
    }

    pub fn reset_last_paused_time(&mut self) {
        self.cached.paused.last = 0.0;
    }

    /// Has the timer been started since it was last reset?
    pub fn was_started(&self) -> bool {
        !self.timestamps.is_empty()
    }

    /// Is the timer currently active?
    pub fn is_active(&self) -> bool {
        match self.timestamps.last() {
            Some(timestamp) => timestamp.is_active(),
            None => false,
        }
    }

    /// Is the timer currently paused?
    pub fn is_paused(&self) -> bool {
        match self.timestamps.last() {
            Some(timestamp) => timestamp.is_paused(),
            None => false,
        }
    }

    /// Is the timer currently stopped?
    pub fn is_stopped(&self) -> bool {
        match self.timestamps.last() {
            Some(timestamp) => timestamp.is_stopped(),
            None => false,
        }
    }

    pub fn current_time(&self) -> f32 {
        match self.timestamps.last() {
            Some(timestamp) => (Instant::now() - timestamp.instant).as_secs_f32(),
            None => 0.0,
        }
    }

    pub fn current_active_time(&self) -> f32 {
        if self.is_active() {
            self.current_time() + self.cached.active.last
        } else {
            self.cached.active.last
        }
    }

    pub fn current_stopped_time(&self) -> f32 {
        if self.is_stopped() {
            self.current_time() + self.cached.stopped.last
        } else {
            self.cached.stopped.last
        }
    }

    pub fn current_paused_time(&self) -> f32 {
        if self.is_paused() {
            self.current_time() + self.cached.active.last
        } else {
            self.cached.paused.last
        }
    }

    /// How long the timer has been active in seconds.
    pub fn active_time(&self) -> f32 {
        self.cached.active.saved + self.cached.active.last + self.current_active_time()
    }

    /// How long the timer has been paused in seconds.
    pub fn paused_time(&self) -> f32 {
        self.cached.paused.saved + self.cached.paused.last + self.current_paused_time()
    }

    /// How long the timer has been stopped in seconds.
    pub fn stopped_time(&self) -> f32 {
        self.cached.stopped.saved + self.cached.stopped.last + self.current_stopped_time()
    }

    /// Sum of current time and all cached times.
    pub fn total_time(&self) -> f32 {
        self.current_time()
            + self.cached.active.saved
            + self.cached.paused.saved
            + self.cached.stopped.saved
            + self.cached.active.last
            + self.cached.paused.last
            + self.cached.stopped.last
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

pub fn view_stopwatch_ms(ui: &mut Ui, timer: &Timer) {
    let t = timer.active_time() + timer.cached.active.last;
    if !timer.was_started() {
        timer_display_ms!(ui, t);
    } else {
        timer_display_ms!(ui, t, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint()
        }
    }
}

pub fn view_stopwatch_hms(ui: &mut Ui, timer: &Timer) {
    let t = timer.active_time() + timer.cached.active.last;
    if !timer.was_started() {
        timer_display_hms!(ui, t);
    } else {
        timer_display_hms!(ui, t, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint()
        }
    }
}

pub fn view_paused_timer(ui: &mut Ui, timer: &Timer) {
    let t = timer.paused_time() + timer.cached.paused.last;
    if !timer.was_started() {
        timer_display_ms!(ui, t);
    } else {
        timer_display_ms!(ui, t, ACTIVE_COLOR);
        if timer.is_paused() {
            ui.request_repaint()
        }
    }
}

pub fn view_total_time_ms(ui: &mut Ui, timer: &Timer) {
    let t = timer.total_time();
    if !timer.was_started() {
        timer_display_ms!(ui, t);
    } else {
        ui.request_repaint();
        timer_display_ms!(ui, t, ACTIVE_COLOR);
    }
}

pub fn view_countdown_ms(ui: &mut Ui, timer: &Timer, countdown_from: f32) {
    let t = countdown_from - timer.active_time();
    if !timer.was_started() {
        timer_display_ms!(ui, t);
        return;
    } else {
        if timer.is_active() {
            ui.request_repaint();
        }
        if t.is_sign_positive() {
            timer_display_ms!(ui, t, ACTIVE_COLOR);
        } else {
            timer_display_ms!(ui, t, NEGATIVE_COLOR);
        }
    }
}

// Special timer for session page which counts down to zero and not below.
pub fn view_nonneg_countdown_ms(ui: &mut Ui, timer: &Timer, countdown_from: f32) {
    let t = (countdown_from - timer.active_time()).max(0.0);
    if !timer.was_started() {
        timer_display_ms!(ui, t);
    } else {
        timer_display_ms!(ui, t, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint();
        }
    }
}

pub fn view_countdown_hms(ui: &mut Ui, timer: &Timer, countdown_from: f32) {
    let t = countdown_from - timer.active_time();
    if !timer.was_started() {
        timer_display_hms!(ui, t);
        return;
    } else {
        if timer.is_active() {
            ui.request_repaint();
        }
        if t.is_sign_positive() {
            timer_display_hms!(ui, t, ACTIVE_COLOR);
        } else {
            timer_display_hms!(ui, t, NEGATIVE_COLOR);
        }
    }
}

pub fn view_nonneg_countdown_hms(ui: &mut Ui, timer: &Timer, countdown_from: f32) {
    let t = (countdown_from - timer.active_time()).max(0.0);
    if !timer.was_started() {
        timer_display_hms!(ui, t);
    } else {
        timer_display_hms!(ui, t, ACTIVE_COLOR);
        if timer.is_active() {
            ui.request_repaint();
        }
    }
}
