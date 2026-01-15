use crate::app::image_utils::average_color;
use crate::app::traits::{CatalogStore, ImageIo};
use crate::domain::{Catalog, Tile};
use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn add_tiles<C: CatalogStore, I: ImageIo>(
    catalog_store: &C,
    image_io: &I,
    path: &Path,
) -> AppResult<Vec<Tile>> {
    let mut catalog = catalog_store.load()?;
    let image_paths = collect_image_paths(path)?;

    if image_paths.is_empty() {
        return Err(AppError::InvalidInput("no image files found".to_string()));
    }

    let mut added = Vec::new();
    for image_path in image_paths {
        let image = image_io.read(&image_path)?;
        let avg_color = average_color(&image);
        let tile = Tile {
            id: tile_id_for_path(&image_path),
            path: image_path.clone(),
            avg_color,
        };

        if catalog.add_tile(tile.clone()) {
            added.push(tile);
        }
    }

    catalog_store.save(&catalog)?;
    Ok(added)
}

pub fn list_tiles<C: CatalogStore>(catalog_store: &C) -> AppResult<Catalog> {
    catalog_store.load()
}

pub fn remove_tile<C: CatalogStore>(catalog_store: &C, id: &str) -> AppResult<Tile> {
    let mut catalog = catalog_store.load()?;
    let removed = catalog
        .remove_by_id(id)
        .ok_or_else(|| AppError::CatalogNotFound(id.to_string()))?;
    catalog_store.save(&catalog)?;
    Ok(removed)
}

fn collect_image_paths(path: &Path) -> AppResult<Vec<PathBuf>> {
    if path.is_file() {
        if is_image_path(path) {
            return Ok(vec![path.to_path_buf()]);
        }
        return Err(AppError::InvalidInput(
            "path is not an image file".to_string(),
        ));
    }

    if !path.is_dir() {
        return Err(AppError::InvalidInput(
            "path is not a file or directory".to_string(),
        ));
    }

    let mut images = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let entry_path = entry.path();
        if entry_path.is_file() && is_image_path(entry_path) {
            images.push(entry_path.to_path_buf());
        }
    }

    Ok(images)
}

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif"))
        .unwrap_or(false)
}

fn tile_id_for_path(path: &Path) -> String {
    let bytes = path.to_string_lossy();
    blake3::hash(bytes.as_bytes()).to_hex().to_string()
}
