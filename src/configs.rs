use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf, sync::OnceLock};

pub const HARDCODED_ZOOM: f32 = 1.5;
pub const HARDCODED_ROOT_DIR: &'static str = "C:\\DataProClients";

pub static DEFAULT_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
pub static DEFAULT_ZOOM: OnceLock<f32> = OnceLock::new();

pub const CLIENT_DATA_FILE_NAME: &'static str = "client_data.txt";
pub const ASSESSMENTS_FILE_NAME: &'static str = "assessments.txt";
pub const KSF_FILE_NAME: &'static str = "ksf_data.txt";
pub const SESSION_DATA_FOLDER_NAME: &'static str = "Session Records";
pub const IOA_DATA_FOLDER_NAME: &'static str = "IOA Data";

#[derive(Serialize, Deserialize, Clone)]
pub struct Configs {
    pub zoom: f32,
    pub root_dir: PathBuf,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            zoom: HARDCODED_ZOOM,
            root_dir: HARDCODED_ROOT_DIR.into(),
        }
    }
}

impl Configs {
    pub fn try_from_current_dir() -> Result<Self> {
        if let Ok(path_buf) = std::env::current_dir() {
            let mut file = File::open(&path_buf.join("config.json"))?;
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let configs: Configs = serde_json::from_str(&s)?;
            return Ok(configs);
        };
        Err(anyhow::anyhow!(""))
    }

    pub fn from_current_dir() -> Self {
        Self::try_from_current_dir().unwrap_or_default()
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self).context("unable to create config.json")
    }
}
