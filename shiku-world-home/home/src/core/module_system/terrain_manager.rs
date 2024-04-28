use crate::core::blueprint::def::{Chunk, Gid, LayerKind, TerrainParams};
use crate::core::blueprint::scene::def::CollisionShape;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::ring::RingVec;
use crate::core::{cantor_pair, CantorPair};
use rapier2d::prelude::{ColliderHandle, Polyline, RigidBodyHandle};
use std::collections::{HashMap, HashSet, VecDeque};

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
    position: (i32, i32),
    polyline_to_vertex_map: HashMap<PolyLineId, Vertex>,
    vertex_to_polyline_map: HashMap<Vertex, PolyLineId>,
    surface_vertices: RingVec<Vertex>,
    surface_vertex_to_index_map: HashMap<Vertex, isize>,
    edge_on_surface_set: HashSet<(Vertex, Vertex)>,
}

impl TerrainGridTile {
    pub fn has_edge_on_surface(&self, edge: &(Vertex, Vertex)) -> bool {
        self.edge_on_surface_set.contains(edge)
    }
}

pub struct PolylineBookkeeping {
    terrain_grid: HashMap<(i32, i32), TerrainGridTile>,
    open_vertices: HashMap<Vertex, (i32, i32)>,
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
            open_vertices: HashMap::new(),
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
                        position: (x, y),
                        vertex_to_polyline_map: HashMap::new(),
                        polyline_to_vertex_map: HashMap::new(),
                        edge_on_surface_set: HashSet::new(),
                        surface_vertices: Self::get_vertices_from_collision_shape(
                            x,
                            y,
                            gid_to_collision_shape_map.get(tile_gid),
                        ),
                        surface_vertex_to_index_map: HashMap::new(),
                        gid: *tile_gid,
                    },
                );
            }
        }
        let tile_positions: Vec<(i32, i32)> =
            polyline_bookkeeping.terrain_grid.keys().copied().collect();
        for tile_pos in tile_positions {
            if let Some(mut tile) = polyline_bookkeeping.terrain_grid.remove(&tile_pos) {
                tile.surface_vertices.data.retain(|vertex| {
                    Self::is_vertex_on_surface(
                        vertex,
                        &tile_pos,
                        &polyline_bookkeeping.terrain_grid,
                    )
                });
                for (i, vertex) in tile.surface_vertices.data.iter().enumerate() {
                    polyline_bookkeeping.open_vertices.insert(*vertex, tile_pos);
                    tile.surface_vertex_to_index_map.insert(*vertex, i as isize);
                }
                let closing_edge = [
                    *tile.surface_vertices.last(),
                    *tile.surface_vertices.first(),
                ];
                for window in tile
                    .surface_vertices
                    .data
                    .windows(2)
                    .chain(std::iter::once(closing_edge.as_slice()))
                {
                    match window {
                        [v1, v2] => {
                            if Self::is_edge_on_surface(
                                (*v1, *v2),
                                &tile_pos,
                                &polyline_bookkeeping.terrain_grid,
                            ) {
                                tile.edge_on_surface_set.insert((*v1, *v2));
                            }
                        }
                        &_ => {}
                    }
                }
                polyline_bookkeeping.terrain_grid.insert(tile_pos, tile);
            }
        }
    }

    fn recalc_polylines(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) {
        let open_vertex_option = polyline_bookkeeping.open_vertices.keys().next().copied();
        let mut current_poly_line: VecDeque<Vertex> = VecDeque::new();
        while let Some(open_vertex) = open_vertex_option {
            current_poly_line.push_back(open_vertex);
            if let Some(tile_pos) = polyline_bookkeeping.open_vertices.remove(&open_vertex) {
                if let Some(tile) = polyline_bookkeeping.terrain_grid.get_mut(&tile_pos) {
                    if let Some(vortex_i) = tile.surface_vertex_to_index_map.get(&open_vertex) {
                        Self::add_clockwise_vertices(
                            &mut current_poly_line,
                            tile,
                            &mut polyline_bookkeeping.open_vertices,
                            vortex_i,
                        );
                        Self::add_counterclockwise_vertices(
                            &mut current_poly_line,
                            tile,
                            &mut polyline_bookkeeping.open_vertices,
                            vortex_i,
                        );
                    }
                }
            }
        }
    }

    fn add_clockwise_vertices(
        current_poly_line: &mut VecDeque<Vertex>,
        tile: &TerrainGridTile,
        open_vertices: &mut HashMap<Vertex, (i32, i32)>,
        vortex_i: &isize,
    ) -> isize {
        let mut current_vortex_i = *vortex_i;
        while tile.has_edge_on_surface(&(
            tile.surface_vertices[current_vortex_i],
            tile.surface_vertices[current_vortex_i + 1],
        )) {
            open_vertices.remove(&tile.surface_vertices[current_vortex_i + 1]);
            current_poly_line.push_back(tile.surface_vertices[current_vortex_i + 1]);
            current_vortex_i += 1;
        }
        current_vortex_i
    }

    fn add_counterclockwise_vertices(
        current_poly_line: &mut VecDeque<Vertex>,
        tile: &TerrainGridTile,
        open_vertices: &mut HashMap<Vertex, (i32, i32)>,
        vortex_i: &isize,
    ) -> isize {
        let mut current_vortex_i = *vortex_i;
        while tile.has_edge_on_surface(&(
            tile.surface_vertices[current_vortex_i - 1],
            tile.surface_vertices[current_vortex_i],
        )) {
            open_vertices.remove(&tile.surface_vertices[current_vortex_i - 1]);
            current_poly_line.push_front(tile.surface_vertices[current_vortex_i - 1]);
            current_vortex_i -= 1;
        }
        current_vortex_i
    }

    fn is_edge_on_surface(
        edge: (Vertex, Vertex),
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
    ) -> RingVec<Vertex> {
        RingVec::new(5)
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
