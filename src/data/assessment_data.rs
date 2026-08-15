use anyhow::{Context, Result};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Conditions {
    pub session: u32,
    pub conditions: IndexSet<String>,
}

impl Conditions {
    // TODO: allow generic type for easier to use API
    pub fn new(conditions: Vec<String>) -> Self {
        Self {
            session: 0,
            conditions: IndexSet::from_iter(conditions.into_iter()),
        }
    }

    pub fn new_with_session(session: u32, conditions: Vec<String>) -> Self {
        Self {
            session,
            conditions: IndexSet::from_iter(conditions.into_iter()),
        }
    }

    pub fn first(&self) -> Option<&String> {
        self.conditions.first()
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).context("unable to convert Conditions to json")
    }
}

/// A list of assessments names paired with a list of their conditions.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AssessmentsData(IndexMap<String, Conditions>);

impl Deref for AssessmentsData {
    type Target = IndexMap<String, Conditions>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssessmentsData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AssessmentsData {
    pub fn from_file(file_path: &PathBuf) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(serde_json::from_str(&s)?)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self).context("unable to convert AssessmentsData to json")
    }

    pub fn fa_conditions() -> Self {
        serde_json::from_str(
            r#"{
                "FA": {
                    "session": 0,
                    "conditions": [
                        "Ignore/Alone",
                        "Tangible",
                        "Demand",
                        "Attention",
                        "Toy Play"
                    ]
                }
            }"#,
        )
        .unwrap()
    }
}

// #[test]
// fn create_example_data() {
//     let mut assess = AssessmentsData::default();
//     assess.insert("FA".into(), Conditions::default());
//     println!("{}", assess.to_json().unwrap());
// }
