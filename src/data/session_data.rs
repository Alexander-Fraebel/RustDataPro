use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DataCollectionType {
    #[default]
    Primary,
    Reliability,
}

impl Display for DataCollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataCollectionType::Primary => write!(f, "Primary"),
            DataCollectionType::Reliability => write!(f, "Reliability"),
        }
    }
}

impl DataCollectionType {
    pub fn abbrev(&self) -> &'static str {
        match self {
            DataCollectionType::Primary => "P",
            DataCollectionType::Reliability => "R",
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
    pub data_collection_type: DataCollectionType,
    pub limit_session_length: bool,
    pub maximum_session_length: f32,
    pub limit_total_length: bool,
    pub maximum_total_length: f32,
}

impl SessionData {
    crate::to_and_from_json!(
        self,
        "unable to make SessionData from file",
        "unable to convert SessionData to json"
    );
}
