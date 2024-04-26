use crate::core::blueprint::def::{Chunk, Gid, LayerKind, TerrainParams};
use crate::core::blueprint::scene::def::CollisionShape;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::{cantor_pair, CantorPair};
use rapier2d::prelude::{ColliderHandle, Polyline, RigidBodyHandle};
use std::collections::{HashMap, HashSet};

type PolyLineId = u32;
type Vertex = (i32, i32);
struct TerrainPolyLine {
    id: PolyLineId,
    body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    vertices: Vec<Vertex>,
    tiles: HashSet<(usize, usize)>,
}

struct TerrainGridTile {
    gid: Gid,
    polyline_to_vertex_map: HashMap<PolyLineId, Vertex>,
    vertex_to_polyline_map: HashMap<Vertex, PolyLineId>,
    surface_vertices: Vec<Vertex>,
    edge_on_surface_set: HashSet<(Vertex, Vertex)>,
}

struct PolylineBookkeeping {
    terrain_grid: HashMap<(i32, i32), TerrainGridTile>,
    gid_to_chunk_map: HashMap<Gid, HashSet<CantorPair>>,
    chunk_to_lines_map: HashMap<CantorPair, HashSet<PolyLineId>>,
    lines: HashMap<PolyLineId, Polyline>,
    id_gen: u32,
}

pub struct TerrainManager {
    pub layer_data: HashMap<LayerKind, HashMap<CantorPair, Chunk>>,
    pub collision_shape_map: HashMap<Gid, CollisionShape>,
    pub params: TerrainParams,
    pub polyline_bookkeeping: PolylineBookkeeping,
}

impl TerrainManager {
    pub fn new(
        params: TerrainParams,
        layer_data: HashMap<LayerKind, HashMap<CantorPair, Chunk>>,
        collision_shape_map: HashMap<Gid, CollisionShape>,
    ) -> TerrainManager {
        let mut polyline_bookkeeping = PolylineBookkeeping {
            terrain_grid: HashMap::new(),
            id_gen: 0,
            gid_to_chunk_map: HashMap::new(),
            chunk_to_lines_map: HashMap::new(),
            lines: HashMap::new(),
        };

        if let Some(terrain_chunks) = layer_data.get(&LayerKind::Terrain) {
            Self::initialize_terrain_grid(
                terrain_chunks,
                &params,
                &collision_shape_map,
                &mut polyline_bookkeeping,
            )
        }

        TerrainManager {
            params,
            layer_data,
            collision_shape_map,
            polyline_bookkeeping,
        }
    }

    pub fn write_chunk(
        &mut self,
        chunk: &Chunk,
        layer_kind: &LayerKind,
        physics: &mut RapierSimulation,
    ) {
        self.layer_data.get_mut(layer_kind).and_then(|chunk_map| {
            if *layer_kind == LayerKind::Terrain {
                Self::update_terrain_collisions(
                    chunk,
                    chunk_map,
                    &self.collision_shape_map,
                    &mut self.polyline_bookkeeping,
                    physics,
                );
            }
            chunk_map.insert(
                cantor_pair(chunk.position.0, chunk.position.1),
                chunk.clone(),
            )
        });
    }

    pub fn initialize_terrain_grid(
        terrain_chunks: &HashMap<u32, Chunk>,
        terrain_params: &TerrainParams,
        gid_to_collision_shape_map: &HashMap<Gid, CollisionShape>,
        polyline_bookkeeping: &mut PolylineBookkeeping,
    ) {
        for (cantor_pair, chunk) in terrain_chunks {
            let (chunk_x, chunk_y) = chunk.position;
            for (i, tile_gid) in chunk.data.iter().enumerate() {
                if *tile_gid == 0 {
                    continue;
                }
                let (x, y) =
                    Self::get_tile_position(chunk_x, chunk_y, terrain_params.chunk_size, i);
                polyline_bookkeeping
                    .gid_to_chunk_map
                    .entry(*tile_gid)
                    .or_default()
                    .insert(*cantor_pair);
                polyline_bookkeeping.terrain_grid.insert(
                    (x, y),
                    TerrainGridTile {
                        vertex_to_polyline_map: HashMap::new(),
                        polyline_to_vertex_map: HashMap::new(),
                        edge_on_surface_set: HashSet::new(),
                        surface_vertices: Self::get_vertices_from_collision_shape(
                            x,
                            y,
                            gid_to_collision_shape_map.get(tile_gid),
                        ),
                        gid: *tile_gid,
                    },
                );
            }
        }
        for (tile_pos, mut tile) in polyline_bookkeeping.terrain_grid {
            tile.surface_vertices.retain(|vertex| {
                Self::is_vertex_on_surface(vertex, &tile_pos, &polyline_bookkeeping.terrain_grid)
            });
            if tile.surface_vertices.len() >= 2 {
                for [v1, v2] in tile.surface_vertices.windows(2) {
                    if Self::is_edge_on_surface(
                        (v1, v2),
                        &tile_pos,
                        &polyline_bookkeeping.terrain_grid,
                    ) {
                        tile.edge_on_surface_set.insert((*v1, *v2));
                    }
                }
            }
        }
    }

    fn recalc_polylines(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) {
        let open_vertex_list: Vec<Vertex> = Vec::with_capacity(20);
    }

    fn is_edge_on_surface(
        edge: (&Vertex, &Vertex),
        tile_pos: &(i32, i32),
        terrain_grid: &HashMap<(i32, i32), TerrainGridTile>,
    ) -> bool {
        true
    }

    fn is_vertex_on_surface(
        vertex: &Vertex,
        tile_pos: &(i32, i32),
        terrain_grid: &HashMap<(i32, i32), TerrainGridTile>,
    ) -> bool {
        true
    }

    fn get_vertices_from_collision_shape(
        tile_x: i32,
        tile_y: i32,
        collision_shape: Option<&CollisionShape>,
    ) -> Vec<Vertex> {
        Vec::new()
    }

    fn get_tile_position(chunk_x: i32, chunk_y: i32, chunk_size: u32, tile_i: usize) -> (i32, i32) {
        (0, 0)
    }

    pub fn update_terrain_collisions(
        chunk: &Chunk,
        terrain_chunks: &HashMap<u32, Chunk>,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) {
    }
}
