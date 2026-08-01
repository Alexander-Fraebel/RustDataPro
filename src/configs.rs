use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

pub const DEFAULT_ZOOM: f32 = 1.5;
pub const DEFAULT_ROOT_DIRECTORY: &'static str = "C:\\DataProClients";

#[derive(Serialize, Deserialize, Clone)]
pub struct Configs {
    pub zoom: f32,
    pub root_dir: String,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            zoom: DEFAULT_ZOOM,
            root_dir: String::from(DEFAULT_ROOT_DIRECTORY),
        }
    }
}

impl Configs {
    pub fn from_file() -> Result<Self> {
        if let Ok(path_buf) = std::env::current_dir() {
            let mut file = File::open(&path_buf)?;
            let mut s = String::new();
            file.read_to_string(&mut s)?;
            let configs: Configs = serde_json::from_str(&s)?;
            return Ok(configs);
        };
        Err(anyhow::anyhow!(""))
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self).context("unable to create config.json")
    }
}
