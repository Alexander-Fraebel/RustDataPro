use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File, io::Read, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DataType {
    #[default]
    Primary,
    Reliability,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Primary => write!(f, "Primary"),
            DataType::Reliability => write!(f, "Reliability"),
        }
    }
}

impl DataType {
    pub fn abbrev(&self) -> &'static str {
        match self {
            DataType::Primary => "P",
            DataType::Reliability => "R",
        }
    }
}

/// Data needed for running a session
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SessionData {
    pub chosen_assessment: String,
    pub chosen_condition: String,
    pub chosen_ksf_name: String,
    pub therapist: String,
    pub data_collector: String,
    pub data_collecion_type: DataType,
    pub limit_session_length: bool,
    pub maximum_session_length: f32,
    pub limit_total_length: bool,
    pub maximum_total_length: f32,
}

impl SessionData {
    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).context("unable to convert session data to json")
    }
}
