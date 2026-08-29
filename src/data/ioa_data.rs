use crate::data::Ksf;
use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Summary of IOA data.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct IoaData {
    pub ten_sec_interval: IndexMap<Key, f32>,
    pub sixty_sec_interval: IndexMap<Key, f32>,
    pub total_duration: IndexMap<Key, f32>,
    pub total_count: IndexMap<Key, f32>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub normalized: bool,
}

impl IoaData {
    pub fn from_ksf(ksf: &Ksf) -> Self {
        let mut ioa = IoaData::default();
        let (f, d) = ksf.keys();
        for k in f {
            // Total duration is meaningless for frequency keys but we need this for alignment when writing
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

    /// Normalize the values. Returns an error if called more than once.
    pub fn normalize(&mut self, n: f32) -> Result<()> {
        if !self.normalized {
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
            self.normalized = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "attempted to normalize IoaData after it was already normalize"
            ))
        }
    }

    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        crate::from_file_path!(self, "unable to make IoaData from file", file_path)
    }

    pub fn to_json(&self) -> Result<String> {
        crate::to_json!(self, "unable to convert IoaData to json")
    }
}
