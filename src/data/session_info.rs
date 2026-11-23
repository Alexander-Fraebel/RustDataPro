use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DataCollectionType {
    #[default]
    Primary,
    Reliability,
}

impl TryFrom<&str> for DataCollectionType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        if ["Primary", "Prim", "P", "primary", "prim", "p"].contains(&value) {
            return Ok(Self::Primary);
        }
        if ["Reliability", "Reli", "R", "reliability", "reli", "r"].contains(&value) {
            return Ok(Self::Reliability);
        }
        Err(anyhow::anyhow!(
            "unable to identify DataCollectionType from string"
        ))
    }
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

/// Data needed for running a session that is not in the client file.
#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SessionInfo {
    pub chosen_assessment: String,
    pub chosen_condition: String,
    pub chosen_ksf_name: String,
    pub therapist: String,
    pub data_collector: String,
    pub data_collection_type: DataCollectionType,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub limit_active_time: bool,
    pub max_active_time: f32,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub limit_total_time: bool,
    pub max_total_time: f32,
}

impl SessionInfo {
    crate::to_and_from_json!(self, " SessionInfo");
}
