use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TilesSource {
    Catalog,
    Directory(PathBuf),
}

#[derive(Debug, Clone)]
pub struct MosaicSpec {
    pub input: PathBuf,
    pub output: PathBuf,
    pub tile_size: u32,
    pub tiles_source: TilesSource,
}

#[derive(Debug, Clone)]
pub struct MosaicResult {
    pub output: PathBuf,
    pub tiles_used: usize,
    pub grid_width: u32,
    pub grid_height: u32,
}
