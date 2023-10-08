use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::terrain_gen::TerrainGenTerrainChunk;
use rapier2d::math::Isometry;
use rapier2d::prelude::{ColliderHandle, Real, RigidBodyHandle, Vector};

use crate::core::entity::def::{Entity, EntityId};
use crate::core::entity::physics::PhysicsRigidBody;
use crate::core::entity::render::NoRender;

pub struct Terrain {}
pub type TerrainEntity = Entity<Terrain, PhysicsRigidBody, NoRender>;

impl TerrainEntity {
    pub fn new_entity(
        entity_id: EntityId,
        isometry: Isometry<Real>,
        body_handle: RigidBodyHandle,
        collider_handle: ColliderHandle,
    ) -> TerrainEntity {
        let terrain_entity: TerrainEntity = Entity {
            id: entity_id,
            isometry,
            physics: PhysicsRigidBody {
                body_handle,
                collider_handle,
                velocity: Vector::new(0.0, 0.0),
            },
            render: NoRender {},
            game_state: Terrain {},
            general_object: None,
            is_render_dirty: false,
            is_position_dirty: false,
            parent_entity: None,
        };
        terrain_entity
    }

    pub fn create_terrain_collider(
        chunk: &TerrainGenTerrainChunk,
        physics: &mut RapierSimulation,
    ) -> (RigidBodyHandle, ColliderHandle) {
        physics.add_terrain_chunk(chunk)
    }
}
