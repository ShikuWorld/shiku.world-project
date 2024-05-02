use crate::core::blueprint::def::{Chunk, Gid, LayerKind, TerrainParams};
use crate::core::blueprint::scene::def::CollisionShape;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::ring::RingVec;
use crate::core::{cantor_pair, CantorPair};
use log::{debug, error};
use rapier2d::prelude::{ColliderHandle, Polyline, RigidBodyHandle};
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
    pub vertices: Vec<Vertex>,
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
    polyline_to_edge_map: HashMap<PolyLineId, Edge>,
    edge_to_polyline_map: HashMap<Edge, PolyLineId>,
    vertices: RingVec<Vertex>,
    edge_start_to_edge_i_map: HashMap<Vertex, isize>,
    edge_end_to_edge_i_map: HashMap<Vertex, isize>,
    surface_edges: RingVec<Edge>,
}

impl TerrainGridTile {
    pub fn vertex_in_polygon(vertices: &RingVec<Vertex>, (x, y): Vertex) -> bool {
        let num_vertices = vertices.len() as isize;
        let mut inside = false;

        let (mut p1_x, mut p1_y) = vertices[0];
        let (mut p2_x, mut p2_y);
        for i in 1..=num_vertices {
            p2_x = vertices[i].0;
            p2_y = vertices[i].1;
            if y >= p1_y.min(p2_y) && y <= p1_y.max(p2_y) && x <= p1_x.max(p2_x) {
                if p2_y == p1_y && y == p1_y {
                    return x >= p1_x.min(p2_x) && x <= p1_x.max(p2_x);
                }
                let x_intersection = ((y - p1_y) * (p2_x - p1_x)) / (p2_y - p1_y) + p1_x;
                if x == x_intersection {
                    return true;
                }
                if p1_x == p2_x || x < x_intersection {
                    inside = !inside;
                }
            }
            p1_x = p2_x;
            p1_y = p2_y;
        }

        inside
    }
}

pub struct PolylineBookkeeping {
    terrain_grid: HashMap<TilePosition, TerrainGridTile>,
    open_edges: HashMap<Edge, TilePosition>,
    gid_to_chunk_map: HashMap<Gid, HashSet<CantorPair>>,
    chunk_to_lines_map: HashMap<CantorPair, HashSet<PolyLineId>>,
    lines: HashMap<PolyLineId, TerrainPolyLine>,
    id_gen: PolyLineId,
}

impl PolylineBookkeeping {
    pub fn get_polyline_id(&mut self) -> PolyLineId {
        self.id_gen += 1;
        self.id_gen
    }
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
        physics: &mut RapierSimulation,
    ) -> TerrainManager {
        debug!("Initializing Terrain Manager");
        let mut polyline_bookkeeping = PolylineBookkeeping {
            terrain_grid: HashMap::new(),
            open_edges: HashMap::new(),
            id_gen: 0,
            gid_to_chunk_map: HashMap::new(),
            chunk_to_lines_map: HashMap::new(),
            lines: HashMap::new(),
        };

        if let Some(terrain_chunks) = layer_data.get(&LayerKind::Terrain) {
            debug!("Initializing Terrain Grid");
            Self::initialize_terrain_grid(
                terrain_chunks,
                &params,
                &collision_shape_map,
                &mut polyline_bookkeeping,
            )
        }

        debug!("Calculating polylines");
        Self::calc_polylines(&mut polyline_bookkeeping, physics);

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
                let (x, y) = Self::get_tile_position(
                    chunk_x,
                    chunk_y,
                    terrain_params.chunk_size as i32,
                    i as i32,
                );
                polyline_bookkeeping
                    .gid_to_chunk_map
                    .entry(*tile_gid)
                    .or_default()
                    .insert(*cantor_pair);
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
                        surface_edges: Self::get_edges_from_vertices(&vertices),
                        vertices,
                        edge_end_to_edge_i_map: HashMap::new(),
                        edge_start_to_edge_i_map: HashMap::new(),
                        gid: *tile_gid,
                    },
                );
            }
        }

        debug!("{:?}", polyline_bookkeeping.terrain_grid.keys());
        let tile_positions: Vec<(i32, i32)> =
            polyline_bookkeeping.terrain_grid.keys().copied().collect();
        for tile_pos in tile_positions {
            if let Some(mut tile) = polyline_bookkeeping.terrain_grid.remove(&tile_pos) {
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

    fn calc_polylines(
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) {
        let mut open_edge_option = polyline_bookkeeping.open_edges.keys().next().copied();
        debug!("Open vertices {:?}", polyline_bookkeeping.open_edges);
        while let Some(open_edge) = open_edge_option {
            debug!("Open vertex {:?}", open_edge);
            let mut current_poly_line: TerrainPolyLineBuilder =
                TerrainPolyLineBuilder::new(polyline_bookkeeping.get_polyline_id());
            if let Some(tile_pos) = polyline_bookkeeping.open_edges.remove(&open_edge) {
                println!("right");
                Self::add_vertices(
                    &mut current_poly_line,
                    &tile_pos,
                    &mut polyline_bookkeeping.terrain_grid,
                    &mut polyline_bookkeeping.open_edges,
                    &open_edge,
                    true,
                );
                println!("left");
                Self::add_vertices(
                    &mut current_poly_line,
                    &tile_pos,
                    &mut polyline_bookkeeping.terrain_grid,
                    &mut polyline_bookkeeping.open_edges,
                    &open_edge,
                    false,
                );
            }
            open_edge_option = polyline_bookkeeping.open_edges.keys().next().copied();
            polyline_bookkeeping.lines.insert(
                current_poly_line.id,
                Self::add_poly_to_physics(current_poly_line, physics),
            );
        }
    }

    fn add_poly_to_physics(
        polyline_builder: TerrainPolyLineBuilder,
        physics: &mut RapierSimulation,
    ) -> TerrainPolyLine {
        let mut vertices = Vec::from(polyline_builder.vertices);
        vertices.dedup();
        let (body_handle, collider_handle) = physics.add_polyine(&vertices);
        TerrainPolyLine {
            id: polyline_builder.id,
            vertices,
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
                if e.1 == next_edge.0 {
                    return Some((next_edge, e_i + 1));
                }

                None
            }
        } else {
            |t: &TerrainGridTile, e: &Edge, e_i: isize| {
                let next_edge = t.surface_edges[e_i - 1];
                if e.0 == next_edge.1 {
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
            println!("current tile position {:?}", current_tile_position);
            if let Some(tile) = terrain_grid.get_mut(&current_tile_position) {
                println!("edges {:?}", tile.surface_edges);
                let mut current_e = start_e;
                current_poly.tiles.insert(current_tile_position);

                println!("adding edge {:?}", current_e);
                open_edges.remove(&current_e);
                tile.polyline_to_edge_map.insert(current_poly.id, current_e);
                if tile
                    .edge_to_polyline_map
                    .insert(current_e, current_poly.id)
                    .is_none()
                {
                    println!("adding vertices {:?}", current_e);
                    add_vertices(current_poly, current_e);
                }

                if let Some(edge_i) = get_edge_i_for_tile(tile, &current_e) {
                    let mut current_edge_i = edge_i;

                    while let Some((next_edge, next_edge_i)) =
                        get_next_edge(tile, &current_e, current_edge_i)
                    {
                        current_edge_i = next_edge_i;
                        current_e = next_edge;
                        println!("adding edge {:?}", current_e);
                        open_edges.remove(&current_e);
                        tile.polyline_to_edge_map.insert(current_poly.id, current_e);
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
                println!("Option before {:?}", current_connected_edge_option);
                current_connected_edge_option = Self::get_connected_edge(
                    &current_tile_position,
                    current_e,
                    terrain_grid,
                    clockwise,
                );
                println!("Option after {:?}", current_connected_edge_option);
            } else {
                error!(
                    "Could not find tile, that should not happen! {:?}",
                    current_connected_edge_option
                );
                current_connected_edge_option = None;
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
        debug!("looking for {:?}", position);
        if let Some(tile) = terrain_grid.get(&position) {
            let edge_i_option = if clockwise {
                tile.edge_start_to_edge_i_map.get(&edge.1)
            } else {
                tile.edge_end_to_edge_i_map.get(&edge.0)
            };
            if let Some(edge) = edge_i_option.map(|edge_i| tile.surface_edges[*edge_i]) {
                if tile.edge_to_polyline_map.contains_key(&edge) {
                    debug!("### already contained");
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
                            .push((tile_x + *x as i32, start_y + *y as i32))
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

    pub fn update_terrain_collisions(
        chunk: &Chunk,
        terrain_chunks: &HashMap<u32, Chunk>,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
        polyline_bookkeeping: &mut PolylineBookkeeping,
        physics: &mut RapierSimulation,
    ) {
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
}
