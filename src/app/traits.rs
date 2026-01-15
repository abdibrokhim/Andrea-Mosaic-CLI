use crate::domain::Catalog;
use crate::error::AppResult;
use image::DynamicImage;
use std::path::Path;

pub trait CatalogStore {
    fn load(&self) -> AppResult<Catalog>;
    fn save(&self, catalog: &Catalog) -> AppResult<()>;
}

pub trait ImageIo {
    fn read(&self, path: &Path) -> AppResult<DynamicImage>;
    fn write_rgb(&self, path: &Path, image: &image::RgbImage) -> AppResult<()>;
}
