use crate::core::blueprint::def::{Chunk, GameMap, LayerKind, TerrainParams};
use crate::core::blueprint::ecs::def::{EntityUpdate, ECS};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{ColliderKind, ColliderShape, RigidBodyType};
use crate::core::module_system::error::CreateWorldError;
use crate::core::rapier_simulation::def::RapierSimulation;
use std::collections::HashMap;

pub type WorldId = String;

pub struct World {
    pub world_id: WorldId,
    pub physics: RapierSimulation,
    pub terrain: HashMap<LayerKind, HashMap<u32, Chunk>>,
    pub terrain_params: TerrainParams,
    pub ecs: ECS,
}

impl World {
    pub fn new(game_map: &GameMap) -> Result<World, CreateWorldError> {
        let world_scene = Blueprint::load_scene(game_map.main_scene.clone().into())?;
        let mut ecs = ECS::from(&world_scene);
        let mut physics = RapierSimulation::new();
        Self::init_physics_simulation_from_ecs(&mut ecs, &mut physics);

        Ok(World {
            world_id: game_map.world_id.clone(),
            terrain_params: TerrainParams {
                chunk_size: game_map.chunk_size,
                tile_height: game_map.tile_height,
                tile_width: game_map.tile_width,
            },
            physics,
            terrain: game_map.terrain.clone(),
            ecs,
        })
    }

    fn init_physics_simulation_from_ecs(ecs: &mut ECS, physics: &mut RapierSimulation) {
        Self::create_initial_rigid_bodies(ecs, physics);
        Self::attach_initial_colliders_to_rigid_bodies(ecs, physics);
    }

    fn create_initial_rigid_bodies(ecs: &mut ECS, physics: &mut RapierSimulation) {
        for (entity, rigid_body_type) in &ecs.rigid_body_type {
            if let Some(transform) = &ecs.transforms.get(entity) {
                let rigid_body_handle = match rigid_body_type {
                    RigidBodyType::Dynamic => {
                        physics.add_dynamic_rigid_body(transform.position.0, transform.position.1)
                    }
                    RigidBodyType::Fixed => {
                        physics.add_fixed_rigid_body(transform.position.0, transform.position.1)
                    }
                    RigidBodyType::KinematicPositionBased => physics
                        .add_kinematic_position_based_rigid_body(
                            transform.position.0,
                            transform.position.1,
                        ),
                    RigidBodyType::KinematicVelocityBased => physics
                        .add_kinematic_velocity_based_rigid_body(
                            transform.position.0,
                            transform.position.1,
                        ),
                };
                ecs.rigid_body_handle.insert(*entity, rigid_body_handle);
            }
        }
    }

    fn attach_initial_colliders_to_rigid_bodies(ecs: &mut ECS, physics: &mut RapierSimulation) {
        for (parent_entity, children) in &ecs.game_node_children {
            if let Some(rigid_body_handle) = ecs.rigid_body_handle.get(parent_entity) {
                for child_entity in children {
                    if let Some(child_collider) = ecs.collider.get(child_entity) {
                        let is_sensor = match child_collider.kind {
                            ColliderKind::Solid => false,
                            ColliderKind::Sensor => true,
                        };
                        let child_collider_handle = match child_collider.shape {
                            ColliderShape::Ball(radius) => {
                                physics.create_ball_collider(radius, *rigid_body_handle, is_sensor)
                            }
                            ColliderShape::CapsuleX(half_y, radius) => physics
                                .create_capsule_x_collider(
                                    half_y,
                                    radius,
                                    *rigid_body_handle,
                                    is_sensor,
                                ),
                            ColliderShape::CapsuleY(half_x, radius) => physics
                                .create_capsule_y_collider(
                                    half_x,
                                    radius,
                                    *rigid_body_handle,
                                    is_sensor,
                                ),
                            ColliderShape::Cuboid(half_x, half_y) => physics
                                .create_cuboid_collider(
                                    half_x,
                                    half_y,
                                    *rigid_body_handle,
                                    is_sensor,
                                ),
                        };
                        ecs.collider_handle
                            .insert(*child_entity, child_collider_handle);
                    }
                }
            }
        }
    }

    pub fn apply_admin_entity_update(&mut self, entity_update: EntityUpdate) {
        self.ecs.apply_entity_update(entity_update);
    }
}
