use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cell::LazyCell,
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    path::Path,
};

const LEAF_PAIR_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"\s*\[\s*(".+"),\s*(".+")\s*]"#).unwrap());
const LEAF_PAIR_REPLACE: &'static str = "\n        [$1, $2]";

const NUM_NAME_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"Num([0123456789])").unwrap());
const NUM_NAME_REPLACE: &'static str = "$1";

/// Renames Egui number key names to just the number (which is easier to read) and makes the representation more compact.
fn prettier_json(text: String) -> String {
    let pass1 = LEAF_PAIR_FIND.replace_all(&text, LEAF_PAIR_REPLACE);
    let pass2 = NUM_NAME_FIND.replace_all(&pass1, NUM_NAME_REPLACE);
    pass2.to_string()
}

const NUM_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r#""([0123456789])""#).unwrap());
const NUM_REPLACE: &'static str = "\"Num$1\"";

/// Rename numbers to number key names that Egui will recognize
fn restore_num_names(text: String) -> String {
    NUM_FIND.replace_all(&text, NUM_REPLACE).to_string()
}

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

/// Key Specification File. A list of keybinds divided into Frequency and Duration.
/// All methods return with Frequency information before Duration.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Ksf {
    #[serde(rename(serialize = "frequency", deserialize = "frequency"))]
    pub freq: Vec<(Key, String)>,
    #[serde(rename(serialize = "duration", deserialize = "duration"))]
    pub dura: Vec<(Key, String)>,
}

impl Ksf {
    /// All key/description pairs.
    pub fn pairs(
        &self,
    ) -> (
        impl Iterator<Item = &(Key, String)>,
        impl Iterator<Item = &(Key, String)>,
    ) {
        (self.freq.iter(), self.dura.iter())
    }

    /// All keys.
    pub fn keys(&self) -> (impl Iterator<Item = &Key>, impl Iterator<Item = &Key>) {
        (
            self.freq.iter().map(|(k, _)| k),
            self.dura.iter().map(|(k, _)| k),
        )
    }

    /// All descriptions.
    pub fn descriptions(&self) -> (impl Iterator<Item = &String>, impl Iterator<Item = &String>) {
        (
            self.freq.iter().map(|(_, d)| d),
            self.dura.iter().map(|(_, d)| d),
        )
    }

    /// Create an IndexMap by cloning the contents.
    pub fn create_map(&self) -> IndexMap<Key, String> {
        let (f, d) = self.pairs();
        IndexMap::from_iter(f.chain(d).cloned())
    }

    /// Are all keys unique across both frequency and duration?
    pub fn keys_unique(&self) -> bool {
        let (f, d) = self.keys();
        f.chain(d).all_unique()
    }

    pub fn example_ksf() -> Ksf {
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

    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let ksf: Ksf = serde_json::from_str(&s)?;
        if !ksf.keys_unique() {
            Err(anyhow::anyhow!("KSF contains duplicate keys"))
        } else {
            Ok(ksf)
        }
    }

    pub fn to_json(&self) -> Result<String> {
        let raw_json =
            serde_json::to_string_pretty(&self).context("unable to convert ksf to json")?;
        Ok(raw_json)
    }
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
                    "the KSF named `{name}` uses a key more than once"
                ));
            }
        }
        Ok(())
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        s = restore_num_names(s);
        let ksf: KsfsData = serde_json::from_str(&s)?;
        Ok(ksf)
    }

    pub fn to_json(&self) -> Result<String> {
        let raw_json =
            serde_json::to_string_pretty(&self).context("unable to convert KsfData to json")?;
        Ok(prettier_json(raw_json))
    }

    pub fn initial_file() -> KsfsData {
        let mut data = KsfsData::default();
        data.insert(String::from("Example"), Ksf::example_ksf());
        data
    }
}
