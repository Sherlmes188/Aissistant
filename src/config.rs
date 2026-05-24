use crate::secret;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct AppConfig {
    pub base_url: String,
    #[serde(skip)]
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_api_key: Option<String>,
    pub model: String,
    pub system_prompt: String,
    pub close_to_tray: bool,
    pub hotkey: String,
}

#[derive(Debug, Deserialize)]
struct StoredConfig {
    #[serde(default = "default_base_url")]
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    encrypted_api_key: Option<String>,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_system_prompt")]
    system_prompt: String,
    #[serde(default = "default_close_to_tray")]
    close_to_tray: bool,
    #[serde(default = "default_hotkey")]
    hotkey: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            api_key: String::new(),
            encrypted_api_key: None,
            model: default_model(),
            system_prompt: default_system_prompt(),
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

        let Ok(stored) = serde_json::from_str::<StoredConfig>(&text) else {
            return Self::default();
        };

        let api_key = stored
            .encrypted_api_key
            .as_deref()
            .and_then(|secret| secret::unprotect_text(secret).ok())
            .filter(|value| !value.is_empty())
            .unwrap_or(stored.api_key);

        Self {
            base_url: stored.base_url,
            api_key,
            encrypted_api_key: stored.encrypted_api_key,
            model: stored.model,
            system_prompt: stored.system_prompt,
            close_to_tray: stored.close_to_tray,
            hotkey: stored.hotkey,
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("failed to create config directory")?;
        }

        let mut config = self.clone();
        config.encrypted_api_key = if config.api_key.trim().is_empty() {
            None
        } else {
            Some(secret::protect_text(config.api_key.trim()).context("failed to protect API key")?)
        };

        let text = serde_json::to_string_pretty(&config).context("failed to serialize config")?;
        fs::write(path, text).context("failed to write config")?;
        Ok(())
    }
}

fn config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "Aissistant", "Aissistant")
        .context("failed to find user config directory")?;
    Ok(dirs.config_dir().join("config.json"))
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_system_prompt() -> String {
    "You are a concise AI assistant. Answer clearly and prefer practical examples.".to_string()
}

fn default_close_to_tray() -> bool {
    true
}

fn default_hotkey() -> String {
    "Ctrl+Space".to_string()
}
