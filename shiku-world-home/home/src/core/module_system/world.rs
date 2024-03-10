use std::collections::HashMap;
use crate::core::blueprint::def::{Chunk, GameMap, LayerKind, Scene, TerrainParams};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::module_system::error::CreateWorldError;

pub type WorldId = String;

pub struct World {
    pub world_id: WorldId,
    pub terrain: HashMap<LayerKind, HashMap<u32, Chunk>>,
    pub terrain_params: TerrainParams,
    pub world_scene: Scene,
}

impl World {
    pub fn new(game_map: &GameMap) -> Result<World, CreateWorldError> {
        let world_scene = Blueprint::load_scene(game_map.main_scene.clone().into())?;
        Ok(World {
            world_id: game_map.world_id.clone(),
            terrain_params: TerrainParams {
                chunk_size: game_map.chunk_size,
                tile_height: game_map.tile_height,
                tile_width: game_map.tile_width,
            },
            terrain: game_map.terrain.clone(),
            world_scene
        })
    }
}