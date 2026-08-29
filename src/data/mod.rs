pub mod assessment_data;
pub mod client_data;
pub mod combined_data;
pub mod ioa_data;
pub mod ksf_data;
pub mod output_data;
pub mod session_data;
pub mod timeline;

pub use assessment_data::*;
pub use client_data::*;
pub use combined_data::*;
pub use ioa_data::*;
pub use ksf_data::*;
pub use output_data::*;
pub use session_data::*;
pub use timeline::*;

use regex::Regex;
use std::cell::LazyCell;

const LEAF_PAIR_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"(\s*)\[\s*(".+")\s*,\s*(".+")\s*]"#).unwrap());
const LEAF_PAIR_REPLACE: &'static str = r#"$1[$2, $3]"#;

const NUM_NAME_FIND: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#""Num([0123456789])""#).unwrap());
const NUM_NAME_REPLACE: &'static str = r#""$1""#;

/// Renames Egui number key names to just the number (which is easier to read and more obvious to write) and makes the representation more compact.
pub fn compact_json_leaves(text: &str) -> String {
    let t = LEAF_PAIR_FIND.replace_all(&text, LEAF_PAIR_REPLACE);
    let t = NUM_NAME_FIND.replace_all(&t, NUM_NAME_REPLACE);
    t.to_string()
}

const NUM_FIND: LazyCell<Regex> = LazyCell::new(|| Regex::new(r#""([0123456789])""#).unwrap());
const NUM_REPLACE: &'static str = r#""Num$1""#;

/// Rename numbers to number key names that Egui will recognize
pub fn expand_num_names(text: &str) -> String {
    NUM_FIND.replace_all(&text, NUM_REPLACE).to_string()
}

#[macro_export]
macro_rules! to_and_from_json {
    ($self:expr, $context1:literal, $context2:literal) => {
        pub fn from_file_path(file_path: &Path) -> Result<Self> {
            serde_json::from_str(&crate::data::expand_num_names(&std::fs::read_to_string(
                file_path,
            )?))
            .context($context1)
        }

        pub fn to_json(&self) -> Result<String> {
            Ok(crate::data::compact_json_leaves(
                &serde_json::to_string_pretty(&self).context($context2)?,
            ))
        }
    };
}
