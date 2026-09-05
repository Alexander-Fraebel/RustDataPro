use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Client information that persists between sessions.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ClientData {
    pub name: String,
    pub id: String,
    pub case_manager: String,
    pub primary_therapist: String,
    pub date_of_admission: String,
    pub location: String,
    pub alternate_ksfs_path: String,
    pub alternate_assessments_path: String,
}

impl ClientData {
    /// Number of days since admission.
    pub fn days_since_admission(&self) -> Result<i32> {
        let x = NaiveDate::parse_from_str(&self.date_of_admission, "%m-%d-%Y")?.num_days_from_ce();
        Ok(Local::now().date_naive().num_days_from_ce() - x)
    }

    // Remove all leading and trailing spaces from String fields.
    pub fn trim_all_fields(&mut self) {
        self.name = self.name.trim().to_owned();
        self.id = self.id.trim().to_owned();
        self.case_manager = self.case_manager.trim().to_owned();
        self.primary_therapist = self.primary_therapist.trim().to_owned();
        self.date_of_admission = self.date_of_admission.trim().to_owned();
        self.location = self.location.trim().to_owned();
    }

    crate::to_and_from_json!(
        self,
        "unable to make ClientData from file",
        "unable to convert ClientData to json"
    );
}
