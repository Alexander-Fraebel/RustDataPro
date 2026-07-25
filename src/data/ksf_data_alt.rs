use anyhow::{Context, Result};
use egui::Key;
use indexmap::IndexMap;
use itertools::Itertools;
// use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    // cell::LazyCell,
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    path::Path,
};

// TODO: restore these to make the actual file nicer to read for troubleshooting
// const LEAF_PAIR_FIND: LazyCell<Regex> =
//     LazyCell::new(|| Regex::new(r"    \[\r?\n      (.+),\r?\n      (.+)\n    \]").unwrap());
// const LEAF_PAIR_REPLACE: &'static str = "    [$1, $2]";

// const NUM_NAME_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r"Num([0123456789])").unwrap());
// const NUM_NAME_REPLACE: &'static str = "$1";

// /// Renames Egui number key names to just the number.
// /// Turns the leaf pairs with KSF key and description into a more compact form
// fn prepare_json_for_writing(text: String) -> String {
//     let pass1 = LEAF_PAIR_FIND.replace_all(&text, LEAF_PAIR_REPLACE);
//     let pass2 = NUM_NAME_FIND.replace_all(&pass1, NUM_NAME_REPLACE);
//     pass2.to_string()
// }

// // Must run before trailing comma as this will add trailing commas
// const MISSING_COMMA_FIND: LazyCell<Regex> =
//     LazyCell::new(|| Regex::new(r#"(\[\".+\", \".+\"\])\r?\n"#).unwrap());
// const MISSING_COMMA_REPLACE: &'static str = "$1,\n";

// const NUM_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r#""([0123456789])""#).unwrap());
// const NUM_REPLACE: &'static str = "\"Num$1\"";

// const TRAILING_COMMA_FIND: LazyCell<Regex> =
//     LazyCell::new(|| Regex::new(r",(\r?\n *[\]\}])").unwrap());
// const TRAILING_COMMA_REPLACE: &'static str = "$1";

// /// Rename numbers to number key names that Egui will recognize
// /// Add in missing commas for leaf pairs then remove trailing commas from those lists
// fn prepare_json_for_reading(text: String) -> String {
//     let pass1 = MISSING_COMMA_FIND.replace_all(&text, MISSING_COMMA_REPLACE);
//     let pass2 = NUM_FIND.replace_all(&pass1, NUM_REPLACE);
//     let pass3 = TRAILING_COMMA_FIND.replace_all(&pass2, TRAILING_COMMA_REPLACE);
//     pass3.to_string()
// }

/// Key Specification File. A list of keybinds divided into Frequency and Duration.
/// All methods return with Frequency information before Duration.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct Ksf {
    pub freq: Vec<(Key, String)>,
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

    /// Create an cloned IndexMap from the contents.
    pub fn create_map(&self) -> IndexMap<Key, String> {
        let (f, d) = self.pairs();
        IndexMap::from_iter(f.chain(d).cloned())
    }

    /// Are all keys unique across both frequency and duration?
    pub fn keys_unique(&self) -> bool {
        let (f, d) = self.keys();
        f.chain(d).all_unique()
    }

    pub fn template_ksf() -> Ksf {
        serde_json::from_str(
            r#"{
                "freq": [
                    ["V", "NegVoc"],
                    ["A", "Aggression"],
                    ["M", "Mand"],
                    ["S", "SIB"],
                    ["I", "Instruction"],
                    ["C", "Compliance"]
                ],
                "dura": [
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

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct KsfData {
    pub ksfs: IndexMap<String, Ksf>,
}

impl Deref for KsfData {
    type Target = IndexMap<String, Ksf>;

    fn deref(&self) -> &Self::Target {
        &self.ksfs
    }
}

impl DerefMut for KsfData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ksfs
    }
}

impl KsfData {
    pub fn from_file(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let ksf: KsfData = serde_json::from_str(&s)?;
        Ok(ksf)
    }

    pub fn to_json(&self) -> Result<String> {
        let raw_json =
            serde_json::to_string_pretty(&self).context("unable to convert ksf to json")?;
        Ok(raw_json)
    }

    pub fn initial_file() -> KsfData {
        let mut data = KsfData::default();
        data.insert(String::from("TEMPLATE"), Ksf::template_ksf());
        data
    }
}
