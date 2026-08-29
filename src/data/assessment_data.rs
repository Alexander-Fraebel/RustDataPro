use anyhow::{Context, Result};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Assessment {
    pub session: u32,
    pub preferred_ksf: String,
    pub conditions: IndexSet<String>,
}

impl Default for Assessment {
    fn default() -> Self {
        Self {
            session: 1,
            preferred_ksf: String::new(),
            conditions: Default::default(),
        }
    }
}

impl Assessment {
    pub fn new<I>(conditions: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            session: 1,
            preferred_ksf: String::new(),
            conditions: IndexSet::from_iter(conditions.into_iter()),
        }
    }

    pub fn new_with_session<I>(session: u32, conditions: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            session,
            preferred_ksf: String::new(),
            conditions: IndexSet::from_iter(conditions.into_iter()),
        }
    }

    pub fn first_condition(&self) -> Option<&String> {
        self.conditions.first()
    }

    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        crate::from_file_path!(self, "unable to make Assessment from file", file_path)
    }

    pub fn to_json(&self) -> Result<String> {
        crate::to_json!(self, "unable to convert Assessment to json")
    }

    pub fn example() -> Self {
        serde_json::from_str(
            r#"{
                "session": 1,
                "preferred_ksf": "",
                "conditions": [
                    "Alone",
                    "Tangible",
                    "Demand",
                    "Attention",
                    "Toy Play"
                ]
            }"#,
        )
        .unwrap()
    }
}

/// A list of assessments names paired with a list of their conditions.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct AssessmentsData(IndexMap<String, Assessment>);

impl Deref for AssessmentsData {
    type Target = IndexMap<String, Assessment>;

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
    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        crate::from_file_path!(self, "unable to make AssessmentsData from file", file_path)
    }

    pub fn to_json(&self) -> Result<String> {
        crate::to_json!(self, "unable to convert AssessmentsData to json")
    }

    pub fn example() -> Self {
        let mut map = IndexMap::new();
        map.insert("FA".into(), Assessment::example());
        Self(map)
    }
}

// #[test]
// fn create_example_data() {
//     println!("{}", Assessment::example().to_json().unwrap());
//     println!("{}", AssessmentsData::example().to_json().unwrap());
// }
