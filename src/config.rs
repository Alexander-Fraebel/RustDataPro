use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf, sync::OnceLock};

pub const HARDCODED_ZOOM: f32 = 1.0;
pub const HARDCODED_ROOT_DIR: &'static str = "C:\\DataProClients";

pub static DEFAULT_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();
pub static DEFAULT_ZOOM: OnceLock<f32> = OnceLock::new();

pub const CLIENT_DATA_FILE_NAME: &'static str = "client_data.txt";
pub const ASSESSMENTS_FILE_NAME: &'static str = "assessments.txt";
pub const KSF_FILE_NAME: &'static str = "ksf_data.txt";
pub const CONFIG_FILE_NAME: &'static str = "config.json";
pub const SESSION_DATA_FOLDER_NAME: &'static str = "Session Records";
pub const IOA_DATA_FOLDER_NAME: &'static str = "IOA Data";

fn default_zoom() -> f32 {
    HARDCODED_ZOOM
}

fn default_root_dir() -> PathBuf {
    HARDCODED_ROOT_DIR.into()
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    #[serde(default = "default_root_dir")]
    pub root_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            zoom: HARDCODED_ZOOM,
            root_dir: HARDCODED_ROOT_DIR.into(),
        }
    }
}

impl Config {
    /// Search the directory the program is in for a config file and try to load it
    pub fn try_from_current_dir() -> Result<Self> {
        if let Ok(path_buf) = std::env::current_dir() {
            let mut file = File::open(&path_buf.join(CONFIG_FILE_NAME))?;
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let configs: Config = serde_json::from_str(&s)?;
            return Ok(configs);
        };
        Err(anyhow::anyhow!(
            "current directory could not be accessed while looking for {}",
            CONFIG_FILE_NAME
        ))
    }

    /// Search the directory the program is in for a config file and load it or create the default config
    pub fn from_current_dir() -> Self {
        Self::try_from_current_dir().unwrap_or_default()
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self)
            .context(format!("unable to create {}", CONFIG_FILE_NAME))
    }
}
