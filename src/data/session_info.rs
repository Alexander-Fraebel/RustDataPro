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
pub struct SessionInfo {
    pub chosen_assessment: String,
    pub chosen_condition: String,
    pub chosen_ksf_name: String,
    pub therapist: String,
    pub data_collector: String,
    pub data_collection_type: DataCollectionType,
    pub limit_active_time: bool,
    pub max_active_time: f32,
    pub limit_total_time: bool,
    pub max_total_time: f32,
}

impl SessionInfo {
    crate::to_and_from_json!(
        self,
        "unable to make SessionInfo from file",
        "unable to convert SessionInfo to json"
    );
}
