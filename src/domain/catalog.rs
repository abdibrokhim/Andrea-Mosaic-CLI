use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub id: String,
    pub path: PathBuf,
    pub avg_color: [u8; 3],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub tiles: Vec<Tile>,
}

impl Catalog {
    pub fn add_tile(&mut self, tile: Tile) -> bool {
        if self.tiles.iter().any(|t| t.path == tile.path || t.id == tile.id) {
            return false;
        }
        self.tiles.push(tile);
        true
    }

    pub fn remove_by_id(&mut self, id: &str) -> Option<Tile> {
        let index = self.tiles.iter().position(|t| t.id == id)?;
        Some(self.tiles.remove(index))
    }
}
