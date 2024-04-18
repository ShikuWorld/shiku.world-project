use crate::core::blueprint::def::{Chunk, GameMap, LayerKind, TerrainParams};
use crate::core::blueprint::ecs::def::{Entity, EntityMaps, EntityUpdate, EntityUpdateKind, ECS};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{
    Collider, ColliderKind, ColliderShape, GameNodeKind, RigidBodyType, Transform,
};
use crate::core::module_system::error::CreateWorldError;
use crate::core::rapier_simulation::def::RapierSimulation;
use log::debug;
use rapier2d::prelude::*;
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

    pub fn update(&mut self) {
        self.physics.update();
        for (entity, rigid_body_handle) in self.ecs.entities.rigid_body_handle.iter() {
            if let Some(transform) = self.ecs.entities.transforms.get_mut(entity) {
                let (x, y, r) = self.physics.get_rigid_body_translation(*rigid_body_handle);
                if transform.position.0 != x || transform.position.1 != y || transform.rotation != r
                {
                    transform.position = (x, y);
                    transform.rotation = r;
                    self.ecs.entities.dirty.insert(*entity, true);
                }
            }
        }
    }

    fn init_physics_simulation_from_ecs(ecs: &mut ECS, physics: &mut RapierSimulation) {
        Self::create_initial_rigid_bodies(ecs, physics);
        Self::attach_initial_colliders_to_rigid_bodies(ecs, physics);
    }

    fn create_initial_rigid_bodies(ecs: &mut ECS, physics: &mut RapierSimulation) {
        for (entity, rigid_body_type) in &ecs.entities.rigid_body_type {
            if let Some(transform) = &ecs.entities.transforms.get(entity) {
                let rigid_body_handle =
                    Self::create_rigid_body_from_type(rigid_body_type, transform, physics);
                ecs.entities
                    .rigid_body_handle
                    .insert(*entity, rigid_body_handle);
                debug!("Successfully added rigid body 1");
            }
        }
    }

    fn add_rigid_body_for_entity(entity: &Entity, ecs: &mut ECS, physics: &mut RapierSimulation) {
        if let (Some(rigid_body_type), Some(transform)) = (
            ecs.entities.rigid_body_type.get(entity),
            ecs.entities.transforms.get(entity),
        ) {
            let rigid_body_handle =
                Self::create_rigid_body_from_type(rigid_body_type, transform, physics);
            ecs.entities
                .rigid_body_handle
                .insert(*entity, rigid_body_handle);
            debug!("Successfully added rigid body 2");
        }
    }

    fn create_rigid_body_from_type(
        rigid_body_type: &RigidBodyType,
        transform: &Transform,
        physics: &mut RapierSimulation,
    ) -> RigidBodyHandle {
        match rigid_body_type {
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
        }
    }

    fn attach_colliders_to_entity(entity: &Entity, ecs: &mut ECS, physics: &mut RapierSimulation) {
        if let (Some(children), Some(rigid_body_handle)) = (
            ecs.entities.game_node_children.get(entity),
            ecs.entities.rigid_body_handle.get(entity),
        ) {
            for child_entity in children {
                if let Some(child_collider) = ecs.entities.collider.get(child_entity) {
                    let child_collider_handle =
                        Self::create_collider(child_collider, rigid_body_handle, physics);
                    ecs.entities
                        .collider_handle
                        .insert(*child_entity, child_collider_handle);
                    debug!("Successfully attached collider 1");
                }
            }
        }
    }

    fn attach_collider_to_its_entity(
        parent_entity: &Entity,
        child_entity: &Entity,
        ecs: &mut ECS,
        physics: &mut RapierSimulation,
    ) {
        if let (Some(child_collider), Some(parent_rigid_body_handle)) = (
            ecs.entities.collider.get(child_entity),
            ecs.entities.rigid_body_handle.get(parent_entity),
        ) {
            let child_collider_handle =
                Self::create_collider(child_collider, parent_rigid_body_handle, physics);
            ecs.entities
                .collider_handle
                .insert(*child_entity, child_collider_handle);
            debug!("Successfully attached collider 2");
        }
    }

    fn attach_initial_colliders_to_rigid_bodies(ecs: &mut ECS, physics: &mut RapierSimulation) {
        for (parent_entity, children) in &ecs.entities.game_node_children {
            if let Some(rigid_body_handle) = ecs.entities.rigid_body_handle.get(parent_entity) {
                for child_entity in children {
                    if let Some(child_collider) = ecs.entities.collider.get(child_entity) {
                        let child_collider_handle =
                            Self::create_collider(child_collider, rigid_body_handle, physics);
                        ecs.entities
                            .collider_handle
                            .insert(*child_entity, child_collider_handle);
                        debug!("Successfully attached collider 2");
                    }
                }
            }
        }
    }

    fn create_collider(
        collider: &Collider,
        rigid_body_handle: &RigidBodyHandle,
        physics: &mut RapierSimulation,
    ) -> ColliderHandle {
        let is_sensor = match collider.kind {
            ColliderKind::Solid => false,
            ColliderKind::Sensor => true,
        };
        match collider.shape {
            ColliderShape::Ball(radius) => {
                physics.create_ball_collider(radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::CapsuleX(half_y, radius) => {
                physics.create_capsule_x_collider(half_y, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::CapsuleY(half_x, radius) => {
                physics.create_capsule_y_collider(half_x, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::Cuboid(half_x, half_y) => {
                physics.create_cuboid_collider(half_x, half_y, *rigid_body_handle, is_sensor)
            }
        }
    }

    pub fn apply_admin_entity_update(&mut self, entity_update: EntityUpdate) {
        if let Some(rigid_body_handle) = self.ecs.entities.rigid_body_handle.get(&entity_update.id)
        {
            let entity = &entity_update.id;
            match entity_update.kind {
                EntityUpdateKind::Transform(transform) => {
                    self.physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(transform.position.0, transform.position.1),
                        transform.rotation,
                        *rigid_body_handle,
                    );
                }
                EntityUpdateKind::PositionRotation((x, y, r)) => {
                    self.physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(x, y),
                        r,
                        *rigid_body_handle,
                    );
                }
                EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                    self.physics.remove_rigid_body(*rigid_body_handle);
                    self.ecs.entities.rigid_body_handle.remove(entity);

                    self.ecs
                        .entities
                        .rigid_body_type
                        .insert(*entity, rigid_body_type);
                    Self::add_rigid_body_for_entity(entity, &mut self.ecs, &mut self.physics);
                    Self::attach_colliders_to_entity(entity, &mut self.ecs, &mut self.physics);
                }
                EntityUpdateKind::Gid(_) | EntityUpdateKind::Name(_) => {}
            }
        } else {
            self.ecs.apply_entity_update(entity_update);
        }
    }

    pub fn add_entity(&mut self, parent_entity: Entity, child: &GameNodeKind) -> Entity {
        let entity = ECS::add_child_to_entity(parent_entity, child, &mut self.ecs);
        Self::add_rigid_body_for_entity(&entity, &mut self.ecs, &mut self.physics);
        Self::attach_colliders_to_entity(&entity, &mut self.ecs, &mut self.physics);
        Self::attach_collider_to_its_entity(
            &parent_entity,
            &entity,
            &mut self.ecs,
            &mut self.physics,
        );
        entity
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let mut children_to_delete = Vec::new();
        Self::get_children_to_delete(&mut children_to_delete, &entity, &mut self.ecs.entities);
        self.ecs.entities.remove_entity(entity);
        for child in children_to_delete {
            self.ecs.entities.remove_entity(child);
        }
    }

    pub fn get_children_to_delete(
        children_to_delete: &mut Vec<Entity>,
        entity: &Entity,
        entities: &mut EntityMaps,
    ) {
        for child in entities
            .game_node_children
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Self::get_children_to_delete(children_to_delete, &child, entities);
            children_to_delete.push(child);
        }
    }
}
