pub mod catalog;
pub mod image_utils;
pub mod mosaic;
pub mod traits;

use crate::app::traits::{CatalogStore, ImageIo};
use crate::domain::{Catalog, MosaicResult, MosaicSpec, Tile};
use crate::error::AppResult;
use std::path::Path;

pub struct App<C: CatalogStore, I: ImageIo> {
    catalog_store: C,
    image_io: I,
}

impl<C: CatalogStore, I: ImageIo> App<C, I> {
    pub fn new(catalog_store: C, image_io: I) -> Self {
        Self {
            catalog_store,
            image_io,
        }
    }

    pub fn catalog_add(&self, path: &Path) -> AppResult<Vec<Tile>> {
        catalog::add_tiles(&self.catalog_store, &self.image_io, path)
    }

    pub fn catalog_list(&self) -> AppResult<Catalog> {
        catalog::list_tiles(&self.catalog_store)
    }

    pub fn catalog_remove(&self, id: &str) -> AppResult<Tile> {
        catalog::remove_tile(&self.catalog_store, id)
    }

    pub fn generate_mosaic(&self, spec: &MosaicSpec) -> AppResult<MosaicResult> {
        mosaic::generate_mosaic(&self.catalog_store, &self.image_io, spec)
    }
}
