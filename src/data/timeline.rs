use anyhow::{Context, Result};
use egui::Key;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

/// Sequence of keypresses and their relative to the start of session
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Timeline(Vec<(Key, f32)>);

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
