use anyhow::{Context, Result};
use egui::{InputState, Key, Modifiers};
use itertools::Itertools;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// Round an f32 to one decimal. To be used for rounding times only.
pub fn rounded_f32(n: f32) -> f32 {
    (n * 10.0).trunc() / 10.0
}

/// Detect keys that have been pressed and ignore repeated events.
pub struct ClickedKeys(Vec<(Key, Modifiers)>);

impl ClickedKeys {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.0.iter().map(|(key, _)| key).contains(key)
    }

    pub fn update(&mut self, input: &InputState) {
        self.clear();

        for event in &input.events {
            if let egui::Event::Key {
                key,
                physical_key: _,
                pressed,
                repeat,
                modifiers,
            } = event
            {
                if *repeat {
                    continue;
                }
                if *pressed {
                    self.0.push((*key, *modifiers));
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
