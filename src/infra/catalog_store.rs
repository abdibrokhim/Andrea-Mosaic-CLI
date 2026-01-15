use crate::app::traits::CatalogStore;
use crate::domain::Catalog;
use crate::error::AppResult;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TomlCatalogStore {
    path: PathBuf,
}

impl TomlCatalogStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn ensure_parent_dir(path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

impl CatalogStore for TomlCatalogStore {
    fn load(&self) -> AppResult<Catalog> {
        if !self.path.exists() {
            return Ok(Catalog::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        let catalog = toml::from_str(&contents)?;
        Ok(catalog)
    }

    fn save(&self, catalog: &Catalog) -> AppResult<()> {
        Self::ensure_parent_dir(&self.path)?;
        let contents = toml::to_string_pretty(catalog)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }
}
