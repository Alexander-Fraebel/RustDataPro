use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

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
    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        crate::from_file_path!(self, "unable to make SessionData from file", file_path)
    }

    pub fn to_json(&self) -> Result<String> {
        crate::to_json!(self, "unable to convert SessionData to json")
    }
}
