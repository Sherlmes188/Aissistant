use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    #[serde(default = "default_close_to_tray")]
    pub close_to_tray: bool,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4o-mini".to_string(),
            system_prompt:
                "You are a concise AI assistant. Answer clearly and prefer practical examples."
                    .to_string(),
            close_to_tray: true,
            hotkey: default_hotkey(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let Ok(path) = config_path() else {
            return Self::default();
        };

        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };

        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("failed to create config directory")?;
        }

        let text = serde_json::to_string_pretty(self).context("failed to serialize config")?;
        fs::write(path, text).context("failed to write config")?;
        Ok(())
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "Aissistant", "Aissistant")
        .context("failed to find user config directory")?;
    Ok(dirs.config_dir().join("config.json"))
}

fn default_close_to_tray() -> bool {
    true
}

fn default_hotkey() -> String {
    "Ctrl+Space".to_string()
}
