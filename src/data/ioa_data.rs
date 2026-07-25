use crate::data::Ksf;
use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::Path};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct IoaData {
    pub ten_sec_interval: IndexMap<Key, f32>,
    pub sixty_sec_interval: IndexMap<Key, f32>,
    pub total_duration: IndexMap<Key, f32>,
    pub total_count: IndexMap<Key, f32>,
}

impl IoaData {
    pub fn from_ksf(ksf: &Ksf) -> Self {
        let mut ioa = IoaData::default();
        // Total duration is meaningless for frequency keys but we need this for alignment
        let (f, d) = ksf.keys();
        for k in f {
            ioa.total_duration.insert(*k, f32::NAN);
            ioa.ten_sec_interval.insert(*k, 0.0);
            ioa.sixty_sec_interval.insert(*k, 0.0);
            ioa.total_count.insert(*k, 0.0);
        }
        for k in d {
            ioa.total_duration.insert(*k, 0.0);
            ioa.ten_sec_interval.insert(*k, 0.0);
            ioa.sixty_sec_interval.insert(*k, 0.0);
            ioa.total_count.insert(*k, 0.0);
        }
        ioa
    }

    /// Normalize the values and round them.
    pub fn finalize(&mut self, n: f32) {
        for v in self.ten_sec_interval.values_mut() {
            *v /= n;
        }
        for v in self.sixty_sec_interval.values_mut() {
            *v /= n;
        }
        for v in self.total_duration.values_mut() {
            *v /= n;
        }
        for v in self.total_count.values_mut() {
            *v /= n;
        }
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self).context("unable to convert session data to json")
    }
}
