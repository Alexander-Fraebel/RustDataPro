use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

pub const ALLOWED_KSF_KEYS: [Key; 36] = [
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
    Key::G,
    Key::H,
    Key::I,
    Key::J,
    Key::K,
    Key::L,
    Key::M,
    Key::N,
    Key::O,
    Key::P,
    Key::Q,
    Key::R,
    Key::S,
    Key::T,
    Key::U,
    Key::V,
    Key::W,
    Key::X,
    Key::Y,
    Key::Z,
];

/// Keyboard Setup File. Two list of keybinds with their descriptions: Frequency and Duration.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Ksf {
    #[serde(rename(serialize = "frequency", deserialize = "frequency"))]
    pub freq: Vec<(Key, String)>,
    #[serde(rename(serialize = "duration", deserialize = "duration"))]
    pub dura: Vec<(Key, String)>,
}

impl Ksf {
    /// All frequency (Key, Description) pairs and all duration (Key, Description) pairs.
    pub fn pairs(
        &self,
    ) -> (
        impl Iterator<Item = &(Key, String)>,
        impl Iterator<Item = &(Key, String)>,
    ) {
        (self.freq.iter(), self.dura.iter())
    }

    /// Frequency keys and duration keys.
    pub fn keys(&self) -> (impl Iterator<Item = &Key>, impl Iterator<Item = &Key>) {
        (
            self.freq.iter().map(|(k, _)| k),
            self.dura.iter().map(|(k, _)| k),
        )
    }

    /// Frequency descriptions and duration descriptions.
    pub fn descriptions(&self) -> (impl Iterator<Item = &String>, impl Iterator<Item = &String>) {
        (
            self.freq.iter().map(|(_, d)| d),
            self.dura.iter().map(|(_, d)| d),
        )
    }

    /// Create an IndexMap by cloning the contents of the Ksf.
    pub fn create_map(&self) -> IndexMap<Key, String> {
        let (f, d) = self.pairs();
        IndexMap::from_iter(f.chain(d).cloned())
    }

    /// Are all keys unique across both frequency and duration?
    pub fn keys_unique(&self) -> bool {
        let (f, d) = self.keys();
        f.chain(d).all_unique()
    }

    pub fn example() -> Ksf {
        serde_json::from_str(&super::expand_num_names(
            r#"{
                "frequency": [
                    ["A", "Aggression"],
                    ["D", "Disruption"],
                    ["S", "SIB"],
                    ["I", "Instruction"],
                    ["C", "Compliance"],
                    ["M", "Mand"],
                    ["L", "Elope"],
                    ["V", "Neg Voc"],
                    ["Z", "Inapp Voc"],
                    ["F", "Ind FCR"],
                    ["G", "Prompt FCR"],
                    ["H", "Inacc FCR"]
                ],
                "duration": [
                    ["K", "Toy Engage"],
                    ["4", "Sr+"],
                    ["6", "Sdelta"]
                ]
            }"#,
        ))
        .unwrap()
    }

    crate::to_and_from_json!(
        self,
        "unable to make Ksf from file",
        "unable to convert Ksf to json"
    );
}

/// A map of KSFs kept in insertion order and index by name.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KsfsData(pub IndexMap<String, Ksf>);

impl Deref for KsfsData {
    type Target = IndexMap<String, Ksf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for KsfsData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl KsfsData {
    // Return an error with the name of the first KSF that does not have all unique keys
    pub fn all_keys_unique(&self) -> Result<()> {
        for (name, ksf) in self.iter() {
            if !ksf.keys_unique() {
                return Err(anyhow::anyhow!(
                    "the KSF named `{name}` uses the same key more than once"
                ));
            }
        }
        Ok(())
    }

    crate::to_and_from_json!(
        self,
        "unable to make KsfsData from file",
        "unable to convert KsfsData to json"
    );

    pub fn example() -> KsfsData {
        let mut data = KsfsData::default();
        data.insert(String::from("Example"), Ksf::example());
        data
    }
}
