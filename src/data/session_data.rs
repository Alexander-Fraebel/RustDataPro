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
            DataType::Primary => "Prim",
            DataType::Reliability => "Reli",
        }
    }
}

/// Data needed for running a session
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct SessionData {
    pub chosen_assessment: String,
    pub chosen_condition: String,
    pub chosen_ksf: String,
    pub therapist: String,
    pub data_collector: String,
    pub data_type: DataType,
}

impl Display for SessionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Assessment: {}\nCondition: {}\nKSF: {}\nTherapist: {}\nData Collector: {}\nData Type: {}",
            self.chosen_assessment,
            self.chosen_condition,
            self.chosen_ksf,
            self.therapist,
            self.data_collector,
            self.data_type,
        )
    }
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
