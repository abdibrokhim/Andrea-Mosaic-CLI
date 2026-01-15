use crate::app::image_utils::{average_color, color_distance};
use crate::app::traits::{CatalogStore, ImageIo};
use crate::domain::{Catalog, MosaicResult, MosaicSpec, TilesSource, Tile};
use crate::error::{AppError, AppResult};
use image::{imageops, DynamicImage, GenericImageView, RgbImage};
use std::path::Path;
use walkdir::WalkDir;

struct TileImage {
    avg_color: [u8; 3],
    image: RgbImage,
}

pub fn generate_mosaic<C: CatalogStore, I: ImageIo>(
    catalog_store: &C,
    image_io: &I,
    spec: &MosaicSpec,
) -> AppResult<MosaicResult> {
    if spec.tile_size == 0 {
        return Err(AppError::InvalidInput(
            "tile size must be greater than zero".to_string(),
        ));
    }

    let input = image_io.read(&spec.input)?;
    let grid_width = input.width() / spec.tile_size;
    let grid_height = input.height() / spec.tile_size;

    if grid_width == 0 || grid_height == 0 {
        return Err(AppError::InvalidInput(
            "input image is smaller than the tile size".to_string(),
        ));
    }

    let tiles = match &spec.tiles_source {
        TilesSource::Catalog => build_tiles_from_catalog(catalog_store, image_io, spec.tile_size)?,
        TilesSource::Directory(path) => build_tiles_from_dir(image_io, path, spec.tile_size)?,
    };

    if tiles.is_empty() {
        return Err(AppError::InvalidInput(
            "no tiles available for mosaic generation".to_string(),
        ));
    }

    let mut output = RgbImage::new(grid_width * spec.tile_size, grid_height * spec.tile_size);

    for tile_y in 0..grid_height {
        for tile_x in 0..grid_width {
            let x = tile_x * spec.tile_size;
            let y = tile_y * spec.tile_size;
            let region = input.view(x, y, spec.tile_size, spec.tile_size).to_image();
            let region_img = DynamicImage::ImageRgba8(region);
            let region_avg = average_color(&region_img);

            let best_tile = tiles
                .iter()
                .min_by_key(|tile| color_distance(tile.avg_color, region_avg))
                .expect("tiles are not empty");

            blit_tile(&mut output, &best_tile.image, x, y);
        }
    }

    image_io.write_rgb(&spec.output, &output)?;

    Ok(MosaicResult {
        output: spec.output.clone(),
        tiles_used: tiles.len(),
        grid_width,
        grid_height,
    })
}

fn build_tiles_from_catalog<C: CatalogStore, I: ImageIo>(
    catalog_store: &C,
    image_io: &I,
    tile_size: u32,
) -> AppResult<Vec<TileImage>> {
    let catalog = catalog_store.load()?;
    build_tiles_from_catalog_data(image_io, &catalog, tile_size)
}

fn build_tiles_from_catalog_data<I: ImageIo>(
    image_io: &I,
    catalog: &Catalog,
    tile_size: u32,
) -> AppResult<Vec<TileImage>> {
    let mut tiles = Vec::new();
    for tile in &catalog.tiles {
        tiles.push(load_tile_image(image_io, tile, tile_size)?);
    }
    Ok(tiles)
}

fn build_tiles_from_dir<I: ImageIo>(
    image_io: &I,
    path: &Path,
    tile_size: u32,
) -> AppResult<Vec<TileImage>> {
    let mut tiles = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let entry_path = entry.path();
        if entry_path.is_file() && is_image_path(entry_path) {
            let image = image_io.read(entry_path)?;
            let avg_color = average_color(&image);
            let resized = image.resize_exact(tile_size, tile_size, imageops::FilterType::Triangle);
            tiles.push(TileImage {
                avg_color,
                image: resized.to_rgb8(),
            });
        }
    }

    Ok(tiles)
}

fn load_tile_image<I: ImageIo>(
    image_io: &I,
    tile: &Tile,
    tile_size: u32,
) -> AppResult<TileImage> {
    let image = image_io.read(&tile.path)?;
    let resized = image.resize_exact(tile_size, tile_size, imageops::FilterType::Triangle);
    Ok(TileImage {
        avg_color: tile.avg_color,
        image: resized.to_rgb8(),
    })
}

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif"))
        .unwrap_or(false)
}

fn blit_tile(output: &mut RgbImage, tile: &RgbImage, x: u32, y: u32) {
    for ty in 0..tile.height() {
        for tx in 0..tile.width() {
            let px = tile.get_pixel(tx, ty);
            output.put_pixel(x + tx, y + ty, *px);
        }
    }
}
