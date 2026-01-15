use crate::app::traits::ImageIo;
use crate::error::AppResult;
use image::{DynamicImage, ImageFormat};
use std::path::Path;

pub struct ImageIoImpl;

impl ImageIoImpl {
    pub fn new() -> Self {
        Self
    }
}

impl ImageIo for ImageIoImpl {
    fn read(&self, path: &Path) -> AppResult<DynamicImage> {
        let image = image::open(path)?;
        Ok(image)
    }

    fn write_rgb(&self, path: &Path, image: &image::RgbImage) -> AppResult<()> {
        let dyn_image = DynamicImage::ImageRgb8(image.clone());
        let format = ImageFormat::from_path(path).ok();
        match format {
            Some(fmt) => dyn_image.save_with_format(path, fmt)?,
            None => dyn_image.save(path)?,
        }
        Ok(())
    }
}
