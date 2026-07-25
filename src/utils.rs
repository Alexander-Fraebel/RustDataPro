use chrono::{DateTime, Datelike, Local, Timelike};
use egui::{Color32, InputState, Key, Response, RichText, Ui};
use itertools::Itertools;
use std::{borrow::Cow, collections::HashSet, ffi::OsStr, path::Path};
use win_msgbox::Okay;

/// Round an f32 to one decimal. To be used for rounding times only.
pub fn rounded_f32(n: f32) -> f32 {
    (n * 10.0).trunc() / 10.0
}

/// Weekday Month/Day/Year Hour:Minute
pub fn date_time_string(dt: &DateTime<Local>) -> String {
    format!(
        "{} {}/{}/{} {:02}:{:02}",
        dt.weekday(),
        dt.month(),
        dt.day(),
        dt.year(),
        dt.hour(),
        dt.minute(),
    )
}

/// Quick time stamp as YYYYMMDDhhmm
pub fn time_stamp() -> String {
    let dt = Local::now();
    format!(
        "{:04}{:02}{:02}{:02}{:02}",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
    )
}

/// Detect keys that have been pressed and ignore repeated events.
pub struct ClickedKeys(HashSet<Key>);

impl ClickedKeys {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.0.contains(key)
    }

    pub fn update(&mut self, input: &InputState) {
        self.clear();

        for event in &input.events {
            if let egui::Event::Key {
                key,
                physical_key: _,
                pressed,
                repeat,
                modifiers: _,
            } = event
            {
                if *repeat {
                    continue;
                }
                if *pressed {
                    self.0.insert(*key);
                }
            }
        }
    }
}

pub fn quick_file_name(pathbuf: &Path) -> Cow<'_, str> {
    pathbuf
        .file_name()
        .unwrap_or(&OsStr::new("INVALID FILE NAME"))
        .to_string_lossy()
}

const DEFAULT_LARGE_BUTTOM_DIMS: (f32, f32) = (120.0, 40.0);
pub trait DataProUiElements {
    fn large_button(&mut self, text: &'static str) -> Response;
    fn large_green_button(&mut self, text: &'static str) -> Response;
    fn green_button(&mut self, text: &'static str) -> Response;
    fn large_red_button(&mut self, text: &'static str) -> Response;
    fn red_button(&mut self, text: &'static str) -> Response;
    fn large_blue_button(&mut self, text: &'static str) -> Response;
    fn blue_button(&mut self, text: &'static str) -> Response;
    fn lock_unlock_button(&mut self, condition: &mut bool);
}

macro_rules! simple_custom_button {
    ($ui:expr, $text:ident, $fill:expr) => {
        $ui.add(
            egui::Button::new(RichText::new($text).monospace().color(Color32::BLACK)).fill($fill),
        )
    };
    (large, $ui:expr, $text:ident, $fill:expr) => {
        $ui.add_sized(
            DEFAULT_LARGE_BUTTOM_DIMS,
            egui::Button::new(RichText::new($text).color(Color32::BLACK)).fill($fill),
        )
    };
}

impl DataProUiElements for Ui {
    fn large_button(&mut self, text: &'static str) -> Response {
        self.add_sized(DEFAULT_LARGE_BUTTOM_DIMS, egui::Button::new(text))
    }

    fn large_green_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_GREEN)
    }

    fn green_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_GREEN)
    }

    fn large_red_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_RED)
    }

    fn red_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_RED)
    }

    fn large_blue_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(large, self, text, Color32::LIGHT_BLUE)
    }

    fn blue_button(&mut self, text: &'static str) -> Response {
        simple_custom_button!(self, text, Color32::LIGHT_BLUE)
    }

    /// Small button that shows an unlocked icon when condition is true and a locked icon when condition is false. Toggles condition on click.
    fn lock_unlock_button(&mut self, condition: &mut bool) {
        if *condition {
            if self.small_button("🔓").clicked() {
                *condition = false;
            }
        } else {
            if self.small_button("🔒").clicked() {
                *condition = true;
            }
        }
    }
}

// Create a windows style error dialog
pub fn windows_error_dialog(message: anyhow::Error) {
    win_msgbox::error::<Okay>(&message.chain().map(|e| e.to_string()).join("\n"))
        .title("Error")
        .set_foreground()
        .show()
        .expect("unable to create well dialog box");
}
