mod app;
mod cli;
mod config;
mod domain;
mod error;
mod infra;
mod ui;

use crate::cli::{CatalogCommands, Commands, GenerateArgs};
use crate::domain::{MosaicSpec, TilesSource};
use crate::error::{AppError, AppResult};
use clap::Parser;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let file_config = config::load(cli.config.as_deref())?;

    let catalog_path = cli
        .catalog_path
        .or(file_config.catalog_path.clone())
        .unwrap_or_else(config::default_catalog_path);

    let default_tile_size = cli
        .default_tile_size
        .or(file_config.default_tile_size)
        .unwrap_or(32);

    let app = app::App::new(
        infra::TomlCatalogStore::new(catalog_path),
        infra::ImageIoImpl::new(),
    );

    match cli.command {
        None => {
            ui::run_tui(app, default_tile_size)?;
        }
        Some(Commands::Catalog { command }) => match command {
            CatalogCommands::Add { path } => {
                let added = app.catalog_add(&path)?;
                cli::print_catalog_add(&added);
            }
            CatalogCommands::List => {
                let catalog = app.catalog_list()?;
                cli::print_catalog_list(&catalog);
            }
            CatalogCommands::Remove { id } => {
                let removed = app.catalog_remove(&id)?;
                cli::print_catalog_remove(&removed);
            }
        },
        Some(Commands::Generate(args)) => {
            let generate_config = file_config.generate.clone().unwrap_or_default();
            let spec = build_mosaic_spec(args, generate_config, default_tile_size)?;
            let result = app.generate_mosaic(&spec)?;
            cli::print_generate_result(&result);
        }
    }

    Ok(())
}

fn build_mosaic_spec(
    args: GenerateArgs,
    file_config: config::GenerateConfig,
    default_tile_size: u32,
) -> AppResult<MosaicSpec> {
    let input = args
        .input
        .or(file_config.input)
        .ok_or_else(|| AppError::InvalidInput("input image is required".to_string()))?;

    let output = args
        .output
        .or(file_config.output)
        .ok_or_else(|| AppError::InvalidInput("output path is required".to_string()))?;

    let tile_size = args
        .tile_size
        .or(file_config.tile_size)
        .unwrap_or(default_tile_size);

    let tiles_value = args.tiles.or(file_config.tiles);
    let tiles_source = resolve_tiles_source(tiles_value)?;

    Ok(MosaicSpec {
        input,
        output,
        tile_size,
        tiles_source,
    })
}

fn resolve_tiles_source(value: Option<String>) -> AppResult<TilesSource> {
    match value {
        None => Ok(TilesSource::Catalog),
        Some(value) => {
            if value.eq_ignore_ascii_case("catalog") {
                Ok(TilesSource::Catalog)
            } else {
                Ok(TilesSource::Directory(PathBuf::from(value)))
            }
        }
    }
}
