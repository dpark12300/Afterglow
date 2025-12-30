use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Dark
    }
}

impl ThemeVariant {
    pub fn toggle(self) -> Self {
        match self {
            ThemeVariant::Dark => ThemeVariant::Light,
            ThemeVariant::Light => ThemeVariant::Dark,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub steamgriddb_api_key: Option<String>,
    #[serde(default)]
    pub theme: ThemeVariant,
    pub lutris_database_path: Option<PathBuf>,
    pub lutris_icons_path: Option<PathBuf>,
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        dirs::config_local_dir().map(|p| p.join("afterglow").join("config.json"))
    }

    pub async fn load() -> Self {
        if let Some(path) = Self::path() {
            if let Ok(content) = fs::read_to_string(path).await {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content).await?;
        }
        Ok(())
    }
}
