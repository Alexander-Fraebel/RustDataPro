use crate::utils::quick_file_name;
use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{cell::LazyCell, fs::File, io::Read, path::Path};

const LEAF_PAIR_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r"    \[\r?\n      (.+),\r?\n      (.+)\n    \]").unwrap());
const LEAF_PAIR_REPLACE: &'static str = "    [$1, $2]";

const NUM_NAME_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"Num([0123456789])").unwrap());
const NUM_NAME_REPLACE: &'static str = "$1";

/// Renames Egui number key names to just the number.
/// Turns the leaf pairs with KSF key and description into a more compact form
fn prepare_json_for_writing(text: String) -> String {
    let pass1 = LEAF_PAIR_FIND.replace_all(&text, LEAF_PAIR_REPLACE);
    let pass2 = NUM_NAME_FIND.replace_all(&pass1, NUM_NAME_REPLACE);
    pass2.to_string()
}

// Must run before trailing comma as this will add trailing commas
const MISSING_COMMA_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"(\[\".+\", \".+\"\])\r?\n"#).unwrap());
const MISSING_COMMA_REPLACE: &'static str = "$1,\n";

const NUM_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r#""([0123456789])""#).unwrap());
const NUM_REPLACE: &'static str = "\"Num$1\"";

const TRAILING_COMMA_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r",(\r?\n *[\]\}])").unwrap());
const TRAILING_COMMA_REPLACE: &'static str = "$1";

/// Rename numbers to number key names that Egui will recognize
/// Add in missing commas for leaf pairs then remove trailing commas from those lists
fn prepare_json_for_reading(text: String) -> String {
    let pass1 = MISSING_COMMA_FIND.replace_all(&text, MISSING_COMMA_REPLACE);
    let pass2 = NUM_FIND.replace_all(&pass1, NUM_REPLACE);
    let pass3 = TRAILING_COMMA_FIND.replace_all(&pass2, TRAILING_COMMA_REPLACE);
    pass3.to_string()
}

/// Key Specification File. A list of keybinds divided into Duration and Frequency.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default, Debug)]
pub struct KsfData {
    #[serde(skip_serializing)] // ignore when saving
    #[serde(skip_deserializing)] // ignore when loading (the ::from_file() method will extract a name)
    #[serde(default)]
    pub name: String,
    pub frequency: Vec<(Key, String)>,
    pub duration: Vec<(Key, String)>,
}

impl KsfData {
    /// All key/description pairs. Frequency first.
    pub fn pairs(&self) -> impl Iterator<Item = &(Key, String)> {
        self.frequency.iter().chain(self.duration.iter())
    }

    /// All keys. Frequency first.
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.pairs().map(|(k, _)| k)
    }

    /// All descriptions. Frequency first.
    pub fn descriptions(&self) -> impl Iterator<Item = &String> {
        self.pairs().map(|(_, d)| d)
    }

    /// Create a HashMap from the contents.
    pub fn create_map(&self) -> IndexMap<Key, String> {
        let mut map = IndexMap::with_capacity(self.frequency.len() + self.duration.len());
        for (k, v) in self.pairs() {
            map.insert(*k, v.clone());
        }
        map
    }

    /// Are all keys unique AND all descriptions unique?
    pub fn all_unique(&self) -> bool {
        self.keys().all_unique() && self.descriptions().all_unique()
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let mut ksf: KsfData = serde_json::from_str(&prepare_json_for_reading(s))?;
        ksf.name = quick_file_name(file_path).to_string();
        if !ksf.all_unique() {
            Err(anyhow::anyhow!(
                "KSF contains duplicate keys or duplicate descriptions"
            ))
        } else {
            Ok(ksf)
        }
    }

    pub fn to_json(&self) -> Result<String> {
        let raw_json =
            serde_json::to_string_pretty(&self).context("unable to convert ksf to json")?;
        Ok(prepare_json_for_writing(raw_json))
    }

    pub fn _test_ksf() -> KsfData {
        serde_json::from_str(
            r#"{
                "frequency": [
                    ["V", "NegVoc"],
                    ["A", "Aggression"],
                    ["M", "Mand"],
                    ["S", "SIB"],
                    ["I", "Instruction"],
                    ["C", "Compliance"]
                ],
                "duration": [
                    ["Num4", "Toy Engage"],
                    ["Num1", "Sr+"],
                    ["Num2", "Sdelta"]
                ]
            }"#,
        )
        .unwrap()
    }
}
