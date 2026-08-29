use crate::{
    data::{AssessmentsData, KsfsData, prettier_json_for_ksf},
    utils::{overwrite_file, windows_error_dialog},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const CLIENT_DATA_FILE_NAME: &'static str = "client_data.txt";
pub const ASSESSMENTS_FILE_NAME: &'static str = "assessments.txt";
pub const KSF_FILE_NAME: &'static str = "ksf_data.txt";
pub const CONFIG_FILE_NAME: &'static str = "config.json";
pub const SESSION_DATA_FOLDER_NAME: &'static str = "Session Records";
pub const IOA_DATA_FOLDER_NAME: &'static str = "IOA Data";

pub fn path_to_config_file() -> Result<PathBuf> {
    Ok(std::env::current_dir()
        .context("error accessing current directory")?
        .join(CONFIG_FILE_NAME))
}

fn hardcoded_zoom() -> f32 {
    1.0
}

fn hardcoded_root_dir() -> String {
    String::from("C:\\DataProClients")
}

fn hardcoded_ksfs_data() -> KsfsData {
    KsfsData::example()
}

fn hardcoded_assessments_data() -> AssessmentsData {
    AssessmentsData::example()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "hardcoded_zoom")]
    pub zoom: f32,
    #[serde(default = "hardcoded_root_dir")]
    pub root_dir: String,
    #[serde(default = "hardcoded_ksfs_data")]
    pub default_ksfs_data: KsfsData,
    #[serde(default = "hardcoded_assessments_data")]
    pub default_assessments_data: AssessmentsData,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            zoom: hardcoded_zoom(),
            root_dir: hardcoded_root_dir(),
            default_ksfs_data: hardcoded_ksfs_data(),
            default_assessments_data: hardcoded_assessments_data(),
        }
    }
}

impl Config {
    /// Search the directory the program is in for a config file and try to load it
    pub fn try_from_current_dir() -> Result<Self> {
        if let Ok(path_buf) = std::env::current_dir() {
            let path_to_config = path_buf.join(CONFIG_FILE_NAME);
            if File::open(&path_to_config).is_err() {
                windows_error_dialog(anyhow::anyhow!(
                    "unable to read {}\na default config file will be created at {}",
                    CONFIG_FILE_NAME,
                    path_to_config.to_string_lossy()
                ));
                overwrite_file(Ok(path_to_config.clone()), &Self::default().to_json()?)?;
                return Ok(Self::default());
            }
            Config::from_file_path(&path_to_config)
        } else {
            Err(anyhow::anyhow!(
                "current directory could not be accessed while looking for {}",
                CONFIG_FILE_NAME
            ))
        }
    }

    pub fn from_file_path(file_path: &Path) -> Result<Self> {
        let mut file = File::open(&file_path)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let ksf: Config = serde_json::from_str(&crate::data::restore_num_names_in_ksf(&s))?;
        Ok(ksf)
    }

    pub fn to_json(&self) -> Result<String> {
        let raw_json =
            serde_json::to_string_pretty(&self).context("unable to convert Config to json")?;
        Ok(prettier_json_for_ksf(&raw_json))
    }
}
