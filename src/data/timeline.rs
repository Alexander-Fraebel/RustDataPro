use anyhow::{Context, Result};
use egui::Key;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

/// Sequence of keypresses and their relative to the start of session
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Timeline(Vec<(Key, f32)>);

impl Timeline {
    /// Append another Timeline to this one, assuming that it begins immediately after this one ends.
    pub fn append(&mut self, other: &Timeline) {
        let offset = self.last().unwrap_or(&(Key::Space, 0.0)).1;
        for item in other.iter() {
            self.push((item.0, item.1 + offset));
        }
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
