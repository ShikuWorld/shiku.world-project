use crate::resource_module::map::def::Layer;
use rapier2d::prelude::Real;

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct TerrainGenTerrainChunk {
    pub x: Real,
    pub y: Real,
    pub tiles_in_x: Real,
    pub tiles_in_y: Real,
    pub tile_width: Real,
    pub tile_height: Real,
}

#[derive(Debug)]
pub struct TerrainTile {
    pub x: Real,
    pub y: Real,
    pub id: u32,
}

pub fn condense_terrain_from_tiles(layer: &Layer) -> Vec<TerrainGenTerrainChunk> {
    let mut chunks = Vec::new();
    let mut tile_map: HashMap<(i32, i32), u32> = HashMap::new();
    let mut tile_not_visited: HashSet<(i32, i32)> = HashSet::new();
    let mut current_open_tile_list: Vec<(i32, i32)> = Vec::new();

    let mut chunk_tile_width: Real = 0.0;
    let mut chunk_tile_height: Real = 0.0;

    for chunk in &layer.terrain_chunks {
        chunk_tile_height = chunk.height;
        chunk_tile_width = chunk.width;

        for (y, row) in chunk.tile_ids.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile != 0 {
                    tile_map.insert(
                        (chunk.x as i32 + x as i32, chunk.y as i32 + y as i32),
                        *tile,
                    );
                    tile_not_visited.insert((chunk.x as i32 + x as i32, chunk.y as i32 + y as i32));
                }
            }
        }
    }

    while !tile_not_visited.is_empty() {
        let tile = tile_not_visited.iter().next().unwrap().clone();
        current_open_tile_list.push(tile);
        let mut chunk = TerrainGenTerrainChunk {
            x: f32::MAX,
            y: f32::MAX,
            tiles_in_x: 0.0,
            tiles_in_y: 1.0,
            tile_width: chunk_tile_width,
            tile_height: chunk_tile_height,
        };

        while !current_open_tile_list.is_empty() {
            if let Some(tile) = current_open_tile_list.pop() {
                if !tile_not_visited.contains(&tile) {
                    continue;
                }

                tile_not_visited.remove(&tile);

                if let Some(_id) = tile_map.get(&tile) {
                    chunk.tiles_in_x += 1.0;
                    chunk.x = chunk.x.min(tile.0 as f32);
                    chunk.y = chunk.y.min(tile.1 as f32);
                }

                let left = (tile.0 - 1, tile.1);
                if tile_map.get(&left).is_some() && tile_not_visited.contains(&left) {
                    current_open_tile_list.push(left);
                }

                let right = (tile.0 + 1, tile.1);
                if tile_map.get(&right).is_some() && tile_not_visited.contains(&right) {
                    current_open_tile_list.push(right);
                }
            }
        }

        chunk.x += (chunk.tiles_in_x - 1.0) / 2.0;
        chunk.y += (chunk.tiles_in_y - 1.0) / 2.0;

        chunks.push(chunk)
    }

    chunks
}
