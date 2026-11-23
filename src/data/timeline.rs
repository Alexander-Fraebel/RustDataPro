use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

use crate::data::Ksf;

/// Sequence of keypresses and their relative to the start of session
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Timeline(Vec<(Key, f32)>);

impl Timeline {
    /// Build a Timeline from a string slice formatted so that each line is formatted as: KEY,TIME
    pub fn from_text(s: &str) -> Result<Timeline> {
        let mut out = Self::default();
        for line in s.lines() {
            let (k, time) = line
                .split_once(",")
                .context("invalid line found when creating Timeline, missing comma")?;

            let key = Key::from_name(k.trim()).context(format!(
                "invalid key name {} found when creating Timeline",
                k.trim(),
            ))?;

            // Remove the previous keypress when Backspace is detected
            // Assume Backspace only appears when valid
            if key == Key::Backspace && !out.is_empty() {
                out.pop();
            }

            out.push((
                key,
                time.trim()
                    .parse()
                    .context("invalid time value found when creating timeline")?,
            ));
        }
        Ok(out)
    }

    /// Marge another Timeline to this one, assuming that it begins immediately after this one ends.
    pub fn merge(&mut self, other: &Timeline) {
        let offset = self.last().unwrap_or(&(Key::Space, 0.0)).1;
        for item in other.iter() {
            self.push((item.0, item.1 + offset));
        }
    }

    /// Caculate summary of the frequency data.
    pub fn calculate_frequency_data(&self, ksf: &Ksf) -> IndexMap<Key, u32> {
        todo!()
    }

    /// Caculate summary of the duration data.
    pub fn calculate_duration_data(&self, ksf: &Ksf) -> IndexMap<Key, u32> {
        todo!()
    }

    /// Return (Total Time, Active Time, Pause Time) as implied by the Timeline.
    pub fn calculate_times(&self) -> (f32, f32, f32) {
        todo!()
    }
}

impl Deref for Timeline {
    type Target = Vec<(Key, f32)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Timeline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Timeline {
    crate::to_and_from_json!(
        self,
        "unable to make Timeline from file",
        "unable to convert Timeline to json"
    );
}
