use crate::error::{AppError, AppResult};
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    pub catalog_path: Option<PathBuf>,
    pub default_tile_size: Option<u32>,
    pub generate: Option<GenerateConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GenerateConfig {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub tiles: Option<String>,
    pub tile_size: Option<u32>,
}

pub fn load(path: Option<&Path>) -> AppResult<FileConfig> {
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };

    if !path.exists() {
        return Err(AppError::ConfigMissing(path.display().to_string()));
    }

    let contents = fs::read_to_string(path)?;
    let config = toml::from_str(&contents)?;
    Ok(config)
}

pub fn default_catalog_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("com", "andreamosaic", "andreamosaic") {
        return project_dirs.data_dir().join("catalog.toml");
    }
    PathBuf::from("catalog.toml")
}
