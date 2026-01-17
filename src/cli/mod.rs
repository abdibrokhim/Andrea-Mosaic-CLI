use crate::domain::{Catalog, MosaicResult, Tile};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "andreamosaic", version, about = "Andrea Mosaic CLI")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub catalog_path: Option<PathBuf>,
    #[arg(long)]
    pub default_tile_size: Option<u32>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Catalog {
        #[command(subcommand)]
        command: CatalogCommands,
    },
    Generate(GenerateArgs),
}

#[derive(Subcommand)]
pub enum CatalogCommands {
    Add { path: PathBuf },
    List,
    Remove { id: String },
}

#[derive(Args)]
pub struct GenerateArgs {
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub tiles: Option<String>,
    #[arg(long)]
    pub tile_size: Option<u32>,
}

pub fn print_catalog_add(added: &[Tile]) {
    if added.is_empty() {
        println!("No new tiles added.");
        return;
    }

    println!("Added {} tile(s):", added.len());
    for tile in added {
        println!("{}  {}", tile.id, tile.path.display());
    }
}

pub fn print_catalog_list(catalog: &Catalog) {
    if catalog.tiles.is_empty() {
        println!("Catalog is empty.");
        return;
    }

    println!("Catalog tiles ({}):", catalog.tiles.len());
    for tile in &catalog.tiles {
        println!("{}  {}", tile.id, tile.path.display());
    }
}

pub fn print_catalog_remove(removed: &Tile) {
    println!("Removed tile {} ({})", removed.id, removed.path.display());
}

pub fn print_generate_result(result: &MosaicResult) {
    println!("Mosaic generated at {}", result.output.display());
    println!("Grid: {} x {}", result.grid_width, result.grid_height);
    println!("Tiles used: {}", result.tiles_used);
}
