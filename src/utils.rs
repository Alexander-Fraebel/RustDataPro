use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, Timelike};
use egui::{InputState, Key};
use itertools::Itertools;
use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

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

pub fn overwrite_file(pathbuf: Result<PathBuf>, data: &str) -> Result<()> {
    match pathbuf {
        Ok(pb) => {
            if pb.exists() {
                std::fs::write(pb, data)?
            } else {
                let mut writer = BufWriter::new(
                    File::create_new(&pb)
                        .with_context(|| format!("error creating file named: {:?}", pb))?,
                );
                writer.write_all(data.as_bytes())?;
                writer.flush()?;
            }
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

// Create a windows style error dialog
pub fn windows_error_dialog(message: anyhow::Error) {
    win_msgbox::error::<win_msgbox::Okay>(&message.chain().map(|e| e.to_string()).join("\n"))
        .title("Error")
        .set_foreground()
        .show()
        .expect("unable to create dialog box");
}

// Ask if the user is sure. Return true if they click Yes and return false if they click No.
pub fn are_you_sure_dialog(message: &str) -> bool {
    win_msgbox::warning::<win_msgbox::YesNo>(message)
        .title("Are you sure?")
        .set_foreground()
        .show()
        .expect("unable to create dialog box")
        == win_msgbox::YesNo::Yes
}

/// Pop up an error dialog if the Result is Err while ignoring Ok.
#[macro_export]
macro_rules! quick_error {
    ($result:expr) => {
        if let Err(e) = $result {
            win_msgbox::error::<win_msgbox::Okay>(&itertools::Itertools::join(
                &mut e.chain().map(|e| e.to_string()),
                "\n",
            ))
            .title("Error")
            .set_foreground()
            .show()
            .expect("unable to create dialog box");
        }
    };
}
