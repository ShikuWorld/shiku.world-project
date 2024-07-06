use crate::core::blueprint::def::{Chunk, Gid, LayerKind, TerrainParams};
use crate::core::blueprint::scene::def::CollisionShape;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::ring::RingVec;
use crate::core::{cantor_pair, CantorPair};
use log::{debug, error};
use rapier2d::math::Real;
use rapier2d::na::Point2;
use rapier2d::prelude::{ColliderHandle, RigidBodyHandle};
use std::collections::{HashMap, HashSet, VecDeque};

type PolyLineId = u32;
type Vertex = (i32, i32);
type Edge = (Vertex, Vertex);
type TilePosition = (i32, i32);

#[derive(Debug)]
pub struct TerrainPolyLine {
    pub id: PolyLineId,
    pub body_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,
    pub original_vertices: VecDeque<Vertex>,
    pub vertices: Vec<Point2<Real>>,
    pub tiles: HashSet<TilePosition>,
}

#[derive(Debug)]
pub struct TerrainPolyLineBuilder {
    pub id: PolyLineId,
    pub vertices: VecDeque<Vertex>,
    pub tiles: HashSet<TilePosition>,
}

impl TerrainPolyLineBuilder {
    pub fn new(id: PolyLineId) -> TerrainPolyLineBuilder {
        TerrainPolyLineBuilder {
            id,
            tiles: HashSet::new(),
            vertices: VecDeque::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct TerrainGridTile {
    gid: Gid,
    position: TilePosition,
    polyline_to_edge_map: HashMap<PolyLineId, HashSet<Edge>>,
    edge_to_polyline_map: HashMap<Edge, PolyLineId>,
    vertices: RingVec<Vertex>,
    edge_start_to_edge_i_map: HashMap<Vertex, isize>,
    edge_end_to_edge_i_map: HashMap<Vertex, isize>,
    surface_edges: RingVec<Edge>,
}

impl TerrainGridTile {
    pub fn vertex_in_polygon(vertices: &RingVec<Vertex>, (x, y): Vertex) -> bool {
        /*println!(
            "Checking if point {:?} is inside polygon {:?}",
            (x, y),
            vertices
        );*/
        let min_x = *vertices.data.iter().map(|(x, _)| x).min().unwrap();
        let max_x = *vertices.data.iter().map(|(x, _)| x).max().unwrap();

        if x < min_x || x > max_x {
            return false;
        }

        let num_vertices = vertices.len() as isize;
        let mut inside = false;

        let (mut p1_x, mut p1_y) = vertices[0];
        let (mut p2_x, mut p2_y);
        for i in 1..=num_vertices {
            p2_x = vertices[i].0;
            p2_y = vertices[i].1;
            //println!("current_edge: {:?} - {:?}", (p1_x, p1_y), (p2_x, p2_y));
            if y >= p1_y.min(p2_y) && y <= p1_y.max(p2_y) && x <= p1_x.max(p2_x) {
                if p2_y == p1_y && y == p1_y {
                    //println!("parallel");
                    return x >= p1_x.min(p2_x) && x <= p1_x.max(p2_x);
                }
                let x_intersection = ((y - p1_y) * (p2_x - p1_x)) / (p2_y - p1_y) + p1_x;
                if x == x_intersection {
                    //println!("exactly on x inter");
                    return true;
                }

                if p1_x == p2_x || x < x_intersection {
                    //println!("inserting intersection point {:?}", (x_intersection, y));
                    inside = !inside;
                }
            }
            p1_x = p2_x;
            p1_y = p2_y;
        }

        if inside {
            //println!("Point {:?} is inside polygon {:?}", (x, y), vertices);
        }

        inside
    }
}

pub struct PolylineBookkeeping {
    terrain_grid: HashMap<TilePosition, TerrainGridTile>,
    pub open_edges: HashMap<Edge, TilePosition>,
    pub gid_to_cantor_pair_map: HashMap<Gid, HashSet<CantorPair>>,
    pub chunk_to_cantor_pair_map: HashMap<CantorPair, HashSet<Gid>>,
    pub lines: HashMap<PolyLineId, TerrainPolyLine>,
    pub id_gen: PolyLineId,
}

impl PolylineBookkeeping {
    pub fn get_polyline_id(&mut self) -> PolyLineId {
        self.id_gen += 1;
        self.id_gen
    }
}

pub struct TerrainManager {
    pub layer_data: HashMap<LayerKind, HashMap<CantorPair, Chunk>>,
    pub params: TerrainParams,
    pub layer_parallax: HashMap<LayerKind, (f32, f32)>,
    pub polyline_bookkeeping: PolylineBookkeeping,
    pub pixel_to_meter_conversion: Real,
}

impl TerrainManager {
    pub fn new(
        params: TerrainParams,
        layer_data: HashMap<LayerKind, HashMap<CantorPair, Chunk>>,
        parallax_map: HashMap<LayerKind, (f32, f32)>,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
        physics: &mut RapierSimulation,
    ) -> TerrainManager {
        debug!("Initializing Terrain Manager");
        let mut polyline_bookkeeping = PolylineBookkeeping {
            terrain_grid: HashMap::new(),
            open_edges: HashMap::new(),
            id_gen: 0,
            gid_to_cantor_pair_map: HashMap::new(),
            chunk_to_cantor_pair_map: HashMap::new(),
            lines: HashMap::new(),
        };

        if let Some(terrain_chunks) = layer_data.get(&LayerKind::Terrain) {
            debug!("Initializing Terrain Grid");
            Self::initialize_terrain_grid(
                terrain_chunks,
                &params,
                collision_shape_map,
                &mut polyline_bookkeeping,
            )
        }
        let pixel_to_meter_conversion = 32.0;
        debug!("Calculating polylines");
        Self::calc_polylines(
            &mut polyline_bookkeeping,
            physics,
            pixel_to_meter_conversion,
        );

        TerrainManager {
            params,
            layer_parallax: parallax_map,
            layer_data,
            polyline_bookkeeping,
            pixel_to_meter_conversion,
        }
    }

    pub fn re_add_polylines(&mut self, physics: &mut RapierSimulation) {
        let mut new_lines = HashMap::new();
        for (id, polyline) in self.polyline_bookkeeping.lines.drain() {
            new_lines.insert(
                id,
                Self::add_poly_to_physics(
                    TerrainPolyLineBuilder {
                        tiles: polyline.tiles,
                        id: polyline.id,
                        vertices: polyline.original_vertices,
                    },
                    physics,
                    self.pixel_to_meter_conversion,
                ),
            );
        }
        self.polyline_bookkeeping.lines = new_lines;
    }

    pub fn update_collision_shape(
        &mut self,
        gid: &Gid,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
        physics: &mut RapierSimulation,
    ) {
        let mut chunks_to_update = Vec::new();
        let terrain_layer = LayerKind::Terrain;
        if let Some(cantor_pairs) = self.polyline_bookkeeping.gid_to_cantor_pair_map.get(gid) {
            if let Some(terrain_chunks) = self.layer_data.get_mut(&terrain_layer) {
                for cantor_pair in cantor_pairs {
                    if let Some(chunk) = terrain_chunks.remove(cantor_pair) {
                        chunks_to_update.push(chunk);
                    }
                }
            }
        }
        for chunk in chunks_to_update {
            self.write_chunk(&chunk, &terrain_layer, collision_shape_map, physics);
        }
    }

    pub fn get_lines_as_vert_vec(&self) -> Vec<Vec<(Real, Real)>> {
        self.polyline_bookkeeping
            .lines
            .values()
            .map(|l| l.vertices.iter().map(|v| (v.x, v.y)).collect())
            .collect()
    }

    pub fn write_chunk(
        &mut self,
        chunk: &Chunk,
        layer_kind: &LayerKind,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
        physics: &mut RapierSimulation,
    ) {
        if let Some(chunk_map) = self.layer_data.get_mut(layer_kind) {
            let chunk_id = cantor_pair(chunk.position.0, chunk.position.1);
            if *layer_kind == LayerKind::Terrain {
                let affected_tile_positions = Self::remove_polylines_of_chunk_and_its_edges(
                    chunk,
                    &self.params,
                    &mut self.polyline_bookkeeping,
                    physics,
                );
                Self::clear_gid_cantor_pair_relations(
                    &mut self.polyline_bookkeeping,
                    &chunk_id,
                    chunk_map.get(&chunk_id),
                );
                chunk_map.insert(chunk_id, chunk.clone());
                Self::add_gid_cantor_pair_relations(
                    &mut self.polyline_bookkeeping,
                    &chunk_id,
                    Some(chunk),
                );
                Self::set_chunk_in_bookkeeping(
                    &self.params,
                    collision_shape_map,
                    &mut self.polyline_bookkeeping,
                    chunk,
                );
                Self::prepare_polyline_calculation_for_tiles(
                    &mut self.polyline_bookkeeping,
                    affected_tile_positions,
                );
                Self::calc_polylines(
                    &mut self.polyline_bookkeeping,
                    physics,
                    self.pixel_to_meter_conversion,
                );
            } else {
                chunk_map.insert(chunk_id, chunk.clone());
            }
        };
    }

    fn clear_gid_cantor_pair_relations(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        chunk_id: &CantorPair,
        chunk_option: Option<&Chunk>,
    ) {
        if let Some(chunk) = chunk_option {
            for tile_gid in &chunk.data {
                polyline_bookkeeping
                    .gid_to_cantor_pair_map
                    .entry(*tile_gid)
                    .or_default()
                    .remove(chunk_id);
                polyline_bookkeeping
                    .chunk_to_cantor_pair_map
                    .entry(*chunk_id)
                    .or_default()
                    .remove(tile_gid);
            }
        }
    }

    fn add_gid_cantor_pair_relations(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        chunk_id: &CantorPair,
        chunk_option: Option<&Chunk>,
    ) {
        if let Some(chunk) = chunk_option {
            for tile_gid in &chunk.data {
                polyline_bookkeeping
                    .gid_to_cantor_pair_map
                    .entry(*tile_gid)
                    .or_default()
                    .insert(*chunk_id);
                polyline_bookkeeping
                    .chunk_to_cantor_pair_map
                    .entry(*chunk_id)
                    .or_default()
                    .insert(*tile_gid);
            }
        }
    }

    pub fn initialize_terrain_grid(
        terrain_chunks: &HashMap<u32, Chunk>,
        terrain_params: &TerrainParams,
        gid_to_collision_shape_map: &HashMap<Gid, CollisionShape>,
        polyline_bookkeeping: &mut PolylineBookkeeping,
    ) {
        for (cantor_pair, chunk) in terrain_chunks {
            Self::add_gid_cantor_pair_relations(polyline_bookkeeping, cantor_pair, Some(&chunk));
            Self::set_chunk_in_bookkeeping(
                terrain_params,
                gid_to_collision_shape_map,
                polyline_bookkeeping,
                chunk,
            );
        }

        let tile_positions: Vec<(i32, i32)> =
            polyline_bookkeeping.terrain_grid.keys().copied().collect();
        Self::prepare_polyline_calculation_for_tiles(polyline_bookkeeping, tile_positions);
    }

    pub fn remove_polylines_of_chunk_and_its_edges(
        chunk: &Chunk,
        terrain_params: &TerrainParams,
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) -> Vec<TilePosition> {
        let mut polyline_set = HashSet::new();
        let mut affected_tile_positions = HashSet::new();
        let (c_x, c_y) = chunk.position;
        let chunk_size = terrain_params.chunk_size as i32;
        let (min_x, max_x) = (c_x * chunk_size - 1, c_x * chunk_size + chunk_size + 1);
        let (min_y, max_y) = (c_y * chunk_size - 1, c_y * chunk_size + chunk_size + 1);
        // Need to remove the lines on the edges as well because new inserted tiles might change
        // how the polylines form
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if (x == min_x || x == max_x) && (y == min_y || y == max_y) {
                    // Ignore edges
                    continue;
                }
                if let Some(tile) = polyline_bookkeeping.terrain_grid.get(&(x, y)) {
                    for p_id in tile.edge_to_polyline_map.values() {
                        polyline_set.insert(*p_id);
                    }
                }
                affected_tile_positions.insert((x, y));
            }
        }

        for p_id in polyline_set {
            if let Some(polyline) = polyline_bookkeeping.lines.remove(&p_id) {
                for pos in polyline.tiles {
                    if let Some(tile) = polyline_bookkeeping.terrain_grid.get_mut(&pos) {
                        if let Some(edges) = tile.polyline_to_edge_map.get(&polyline.id) {
                            for edge in edges {
                                tile.edge_to_polyline_map.remove(edge);

                                // The open edges will automatically be filled in by the chunk
                                // insertion step, we only want to add back to open_edges
                                // for poly-lines that go way out into other chunks
                                if !affected_tile_positions.contains(&pos) {
                                    polyline_bookkeeping.open_edges.insert(*edge, pos);
                                }
                            }
                        }
                    }
                }

                physics.remove_rigid_body(polyline.body_handle);
            }
        }
        affected_tile_positions.into_iter().collect()
    }

    fn prepare_polyline_calculation_for_tiles(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        tile_positions: Vec<(i32, i32)>,
    ) {
        for tile_pos in tile_positions {
            if let Some(mut tile) = polyline_bookkeeping.terrain_grid.remove(&tile_pos) {
                tile.surface_edges = Self::get_edges_from_vertices(&tile.vertices);
                tile.edge_start_to_edge_i_map.clear();
                tile.edge_end_to_edge_i_map.clear();

                tile.surface_edges.data.retain(|edge| {
                    Self::is_edge_on_surface(*edge, &tile_pos, &polyline_bookkeeping.terrain_grid)
                });
                for (i, edge) in tile.surface_edges.data.iter().enumerate() {
                    tile.edge_start_to_edge_i_map.insert(edge.0, i as isize);
                    tile.edge_end_to_edge_i_map.insert(edge.1, i as isize);
                    polyline_bookkeeping.open_edges.insert(*edge, tile_pos);
                }
                polyline_bookkeeping.terrain_grid.insert(tile_pos, tile);
            }
        }
    }

    fn set_chunk_in_bookkeeping(
        terrain_params: &TerrainParams,
        gid_to_collision_shape_map: &HashMap<Gid, CollisionShape>,
        polyline_bookkeeping: &mut PolylineBookkeeping,
        chunk: &Chunk,
    ) {
        let (chunk_x, chunk_y) = chunk.position;
        for (i, tile_gid) in chunk.data.iter().enumerate() {
            let (x, y) = Self::get_tile_position(
                chunk_x,
                chunk_y,
                terrain_params.chunk_size as i32,
                i as i32,
            );
            if *tile_gid == 0 {
                polyline_bookkeeping.terrain_grid.remove(&(x, y));
                continue;
            }
            let vertices = Self::get_vertices_from_collision_shape(
                x,
                y,
                terrain_params,
                gid_to_collision_shape_map.get(tile_gid),
            );
            polyline_bookkeeping.terrain_grid.insert(
                (x, y),
                TerrainGridTile {
                    position: (x, y),
                    edge_to_polyline_map: HashMap::new(),
                    polyline_to_edge_map: HashMap::new(),
                    surface_edges: RingVec::new(8),
                    vertices,
                    edge_end_to_edge_i_map: HashMap::new(),
                    edge_start_to_edge_i_map: HashMap::new(),
                    gid: *tile_gid,
                },
            );
        }
    }

    fn calc_polylines(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
        pixel_to_meter_conversion: Real,
    ) {
        let mut open_edge_option = polyline_bookkeeping.open_edges.keys().next().copied();
        while let Some(open_edge) = open_edge_option {
            if let Some(tile_pos) = polyline_bookkeeping.open_edges.remove(&open_edge) {
                if polyline_bookkeeping.terrain_grid.contains_key(&tile_pos) {
                    let mut current_poly_line: TerrainPolyLineBuilder =
                        TerrainPolyLineBuilder::new(polyline_bookkeeping.get_polyline_id());
                    Self::add_vertices(
                        &mut current_poly_line,
                        &tile_pos,
                        &mut polyline_bookkeeping.terrain_grid,
                        &mut polyline_bookkeeping.open_edges,
                        &open_edge,
                        true,
                    );
                    Self::add_vertices(
                        &mut current_poly_line,
                        &tile_pos,
                        &mut polyline_bookkeeping.terrain_grid,
                        &mut polyline_bookkeeping.open_edges,
                        &open_edge,
                        false,
                    );
                    polyline_bookkeeping.lines.insert(
                        current_poly_line.id,
                        Self::add_poly_to_physics(
                            current_poly_line,
                            physics,
                            pixel_to_meter_conversion,
                        ),
                    );
                }
            }

            open_edge_option = polyline_bookkeeping.open_edges.keys().next().copied();
        }
    }

    fn add_poly_to_physics(
        polyline_builder: TerrainPolyLineBuilder,
        physics: &mut RapierSimulation,
        pixel_to_meter_conversion: Real,
    ) -> TerrainPolyLine {
        let mut vec = Vec::from(polyline_builder.vertices.clone());
        vec.dedup();
        let vertices: Vec<Point2<Real>> = vec
            .into_iter()
            .map(|v| {
                Point2::new(
                    (v.0 as Real) / pixel_to_meter_conversion,
                    (v.1 as Real) / pixel_to_meter_conversion,
                )
            })
            .collect();
        let (body_handle, collider_handle) = physics.add_polyine(vertices.clone());
        TerrainPolyLine {
            id: polyline_builder.id,
            vertices,
            original_vertices: polyline_builder.vertices,
            tiles: polyline_builder.tiles,
            body_handle,
            collider_handle,
        }
    }

    fn add_vertices(
        current_poly: &mut TerrainPolyLineBuilder,
        start_tile_pos: &TilePosition,
        terrain_grid: &mut HashMap<TilePosition, TerrainGridTile>,
        open_edges: &mut HashMap<Edge, TilePosition>,
        start_edge: &Edge,
        clockwise: bool,
    ) {
        let mut current_connected_edge_option = Some((*start_tile_pos, *start_edge));
        let get_next_edge = if clockwise {
            |t: &TerrainGridTile, e: &Edge, e_i: isize| {
                let next_edge = t.surface_edges[e_i + 1];
                if e.1 == next_edge.0 && !t.edge_to_polyline_map.contains_key(&next_edge) {
                    return Some((next_edge, e_i + 1));
                }
                None
            }
        } else {
            |t: &TerrainGridTile, e: &Edge, e_i: isize| {
                let next_edge = t.surface_edges[e_i - 1];
                if e.0 == next_edge.1 && !t.edge_to_polyline_map.contains_key(&next_edge) {
                    return Some((next_edge, e_i - 1));
                }
                None
            }
        };
        let get_edge_i_for_tile = if clockwise {
            |t: &TerrainGridTile, edge: &Edge| t.edge_start_to_edge_i_map.get(&edge.0).copied()
        } else {
            |t: &TerrainGridTile, edge: &Edge| t.edge_end_to_edge_i_map.get(&edge.1).copied()
        };
        let add_vertices = if clockwise {
            |p: &mut TerrainPolyLineBuilder, e: Edge| {
                p.vertices.push_back(e.0);
                p.vertices.push_back(e.1);
            }
        } else {
            |p: &mut TerrainPolyLineBuilder, e: Edge| {
                p.vertices.push_front(e.1);
                p.vertices.push_front(e.0);
            }
        };

        while let Some((current_tile_position, start_e)) = current_connected_edge_option {
            if let Some(tile) = terrain_grid.get_mut(&current_tile_position) {
                let mut current_e = start_e;
                current_poly.tiles.insert(current_tile_position);

                open_edges.remove(&current_e);
                tile.polyline_to_edge_map
                    .entry(current_poly.id)
                    .or_default()
                    .insert(current_e);
                if tile
                    .edge_to_polyline_map
                    .insert(current_e, current_poly.id)
                    .is_none()
                {
                    add_vertices(current_poly, current_e);
                }

                if let Some(edge_i) = get_edge_i_for_tile(tile, &current_e) {
                    let mut current_edge_i = edge_i;

                    while let Some((next_edge, next_edge_i)) =
                        get_next_edge(tile, &current_e, current_edge_i)
                    {
                        current_edge_i = next_edge_i;
                        current_e = next_edge;
                        open_edges.remove(&current_e);
                        tile.polyline_to_edge_map
                            .entry(current_poly.id)
                            .or_default()
                            .insert(current_e);
                        if tile
                            .edge_to_polyline_map
                            .insert(current_e, current_poly.id)
                            .is_none()
                        {
                            add_vertices(current_poly, current_e);
                        }
                    }
                } else {
                    error!(
                        "Could not find vortex in tile, that should not happen! {:?}",
                        current_e
                    );
                }
                current_connected_edge_option = Self::get_connected_edge(
                    &current_tile_position,
                    current_e,
                    terrain_grid,
                    clockwise,
                );
            } else {
                let next_open_edge_key = open_edges.keys().next().cloned();
                current_connected_edge_option =
                    next_open_edge_key.and_then(|k| open_edges.remove(&k).map(|t| (t, k)));
            }
        }
    }

    fn get_connected_edge(
        (c_x, c_y): &TilePosition,
        e: Edge,
        terrain_grid: &HashMap<TilePosition, TerrainGridTile>,
        c: bool,
    ) -> Option<(TilePosition, Edge)> {
        let mut tile_positions = Vec::with_capacity(4);
        Self::add_connected_edge(&e, terrain_grid, &mut tile_positions, (c_x - 1, *c_y), c);
        Self::add_connected_edge(&e, terrain_grid, &mut tile_positions, (c_x + 1, *c_y), c);
        Self::add_connected_edge(&e, terrain_grid, &mut tile_positions, (*c_x, c_y - 1), c);
        Self::add_connected_edge(&e, terrain_grid, &mut tile_positions, (*c_x, c_y + 1), c);

        if tile_positions.len() == 1 {
            return Some(tile_positions[0]);
        }

        if tile_positions.len() > 1 {
            error!(
                "Only one or no connected surrounding tiles should exist, what is this?! {:?}",
                tile_positions
            );
        }

        None
    }

    fn add_connected_edge(
        edge: &Edge,
        terrain_grid: &HashMap<TilePosition, TerrainGridTile>,
        possible_tile_positions: &mut Vec<(TilePosition, Edge)>,
        position: TilePosition,
        clockwise: bool,
    ) {
        if let Some(tile) = terrain_grid.get(&position) {
            let edge_i_option = if clockwise {
                tile.edge_start_to_edge_i_map.get(&edge.1)
            } else {
                tile.edge_end_to_edge_i_map.get(&edge.0)
            };
            if let Some(edge) = edge_i_option.map(|edge_i| tile.surface_edges[*edge_i]) {
                if tile.edge_to_polyline_map.contains_key(&edge) {
                    return;
                }
                possible_tile_positions.push((position, edge));
            }
        }
    }

    fn is_edge_on_surface(
        edge: Edge,
        (x, y): &TilePosition,
        terrain_grid: &HashMap<TilePosition, TerrainGridTile>,
    ) -> bool {
        //debug!("Checking edge {:?} on tile {:?}", edge, (x, y));
        let left_touching = terrain_grid
            .get(&(x - 1, *y))
            .map(|tile| Self::is_surface_touching_polygon(edge, tile))
            .unwrap_or_default();
        let right_touching = terrain_grid
            .get(&(x + 1, *y))
            .map(|tile| Self::is_surface_touching_polygon(edge, tile))
            .unwrap_or_default();
        let top_touching = terrain_grid
            .get(&(*x, y - 1))
            .map(|tile| Self::is_surface_touching_polygon(edge, tile))
            .unwrap_or_default();
        let bottom_touch = terrain_grid
            .get(&(*x, y + 1))
            .map(|tile| Self::is_surface_touching_polygon(edge, tile))
            .unwrap_or_default();
        if left_touching || right_touching || top_touching || bottom_touch {
            /*debug!(
                "Edge {:?} on tile {:?} is touching left: {}, right: {}, top: {}, bottom: {}",
                edge,
                (x, y),
                left_touching,
                right_touching,
                top_touching,
                bottom_touch
            );*/
        }

        !left_touching && !right_touching && !top_touching && !bottom_touch
    }

    fn is_surface_touching_polygon((v1, v2): Edge, tile: &TerrainGridTile) -> bool {
        TerrainGridTile::vertex_in_polygon(&tile.vertices, v1)
            && TerrainGridTile::vertex_in_polygon(&tile.vertices, v2)
    }

    fn get_vertices_from_collision_shape(
        tile_x: i32,
        tile_y: i32,
        terrain_params: &TerrainParams,
        collision_shape_option: Option<&CollisionShape>,
    ) -> RingVec<Vertex> {
        let mut vertices = RingVec::<Vertex>::new(10);
        let (tile_width, tile_height) = (
            terrain_params.tile_width as i32,
            terrain_params.tile_height as i32,
        );
        let (start_x, start_y) = (tile_x * tile_width, tile_y * tile_height);
        if let Some(collision_shape) = collision_shape_option {
            match collision_shape {
                CollisionShape::Rectangle(x, y, w, h) => {
                    vertices
                        .data
                        .push((start_x + *x as i32, start_y + *y as i32));
                    vertices
                        .data
                        .push((start_x + (x + w) as i32, start_y + *y as i32));
                    vertices
                        .data
                        .push((start_x + (x + w) as i32, start_y + (y + h) as i32));
                    vertices
                        .data
                        .push((start_x + *x as i32, start_y + (y + h) as i32));
                }
                CollisionShape::Circle(x, y, r) => {
                    vertices
                        .data
                        .push((start_x + (x - r) as i32, start_y + (y - r) as i32));
                    vertices
                        .data
                        .push((start_x + (x + r) as i32, start_y + (y - r) as i32));
                    vertices
                        .data
                        .push((start_x + (x + r) as i32, start_y + (y + r) as i32));
                    vertices
                        .data
                        .push((start_x + (x - r) as i32, start_y + (y + r) as i32));
                }
                CollisionShape::Polygon(v) => {
                    for (x, y) in v {
                        vertices
                            .data
                            .push((start_x + *x as i32, start_y + *y as i32))
                    }
                }
            }
        } else {
            vertices.data.push((start_x, start_y));
            vertices.data.push((start_x + tile_width, start_y));
            vertices
                .data
                .push((start_x + tile_width, start_y + tile_height));
            vertices.data.push((start_x, start_y + tile_height));
        }

        vertices
    }

    fn get_tile_position(chunk_x: i32, chunk_y: i32, chunk_size: i32, tile_i: i32) -> TilePosition {
        let x = tile_i % chunk_size;
        let y = tile_i / chunk_size;
        ((chunk_x * chunk_size) + x, (chunk_y * chunk_size) + y)
    }
    fn get_edges_from_vertices(vertices: &RingVec<Vertex>) -> RingVec<Edge> {
        let mut edge_list = RingVec::new(100);
        for w in vertices.data.windows(2) {
            match w {
                [v1, v2] => edge_list.data.push((*v1, *v2)),
                &_ => {}
            }
        }
        edge_list.data.push((*vertices.last(), *vertices.first()));
        edge_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_inside_polygon() {
        let vertices = RingVec::from(vec![(0, 0), (0, 5), (5, 5), (5, 0)]);

        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (2, 2)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (0, 0)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (5, 5)));
        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (6, 6)));
        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (-1, -1)));
    }

    #[test]
    fn vertex_in_polygon_bug_1() {
        let vertices = RingVec::from(vec![(9, 0), (25, 0), (25, 16), (9, 16)]);

        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (9, 0)));
        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (8, 0)));
    }

    #[test]
    fn test_point_on_polygon_edge() {
        let vertices = RingVec::from(vec![(0, 0), (5, 0), (5, 5), (0, 5)]);

        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (0, 2)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (2, 5)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (5, 3)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (3, 0)));
    }

    #[test]
    fn test_point_inside_concave_polygon() {
        let vertices = RingVec::from(vec![
            (0, 0),
            (0, 5),
            (2, 5),
            (2, 3),
            (4, 3),
            (4, 5),
            (6, 5),
            (6, 0),
        ]);

        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (1, 2)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (5, 2)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (3, 2)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (2, 4)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (1, 4)));
        assert!(TerrainGridTile::vertex_in_polygon(&vertices, (0, 4)));
        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (7, 3)));
        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (3, 4)));
    }

    #[test]
    fn vertex_in_polygon_bug_2() {
        let vertices = RingVec::from(vec![(32, 2), (32, 16), (16, 16), (16, 3)]);

        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (0, 3)));
    }

    #[test]
    fn vertex_in_polygon_bug_3() {
        let vertices = RingVec::from(vec![(16, 0), (29, 0), (29, 9), (16, 15)]);

        assert!(!TerrainGridTile::vertex_in_polygon(&vertices, (0, 15)));
    }
}
