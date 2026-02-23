use dirs::home_dir;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultsCatalog {
    pub families: Vec<DefaultsFamily>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultsFamily {
    pub base: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub subcommands: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

pub fn defaults_path() -> Option<PathBuf> {
    let mut p = home_dir()?;
    p.push(".sutra");
    std::fs::create_dir_all(&p).ok();
    p.push("defaults.json");
    Some(p)
}

pub fn load_defaults_catalog() -> DefaultsCatalog {
    let Some(p) = defaults_path() else {
        return DefaultsCatalog { families: vec![] };
    };
    if !p.exists() {
        return DefaultsCatalog { families: vec![] };
    }
    let Ok(data) = fs::read_to_string(p) else {
        return DefaultsCatalog { families: vec![] };
    };
    serde_json::from_str(&data).unwrap_or(DefaultsCatalog { families: vec![] })
}