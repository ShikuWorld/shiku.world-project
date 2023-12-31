use std::ops::Deref;

use log::error;
use rapier2d::crossbeam;
use rapier2d::prelude::*;

use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::terrain_gen::TerrainGenTerrainChunk;

pub const COL_GROUP_A: InteractionGroups = InteractionGroups::new(Group::GROUP_1, Group::GROUP_1);
pub const COL_GROUP_B: InteractionGroups = InteractionGroups::new(Group::GROUP_2, Group::GROUP_2);

impl RapierSimulation {
    pub fn update(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            self.events.deref(),
        );
    }

    pub fn get_intersecting_colliders(
        &self,
        collider_handle: ColliderHandle,
    ) -> Vec<ColliderHandle> {
        self.narrow_phase
            .intersections_with(collider_handle)
            .filter(|(c1, c2, intersecting)| *intersecting)
            .map(|(c1, c2, _i)| if c1 == collider_handle { c2 } else { c1 })
            .collect()
    }

    pub fn get_collider_aabb(&self, collider_handle: ColliderHandle) -> Aabb {
        if let Some(collider) = self.colliders.get(collider_handle) {
            collider.compute_aabb()
        } else {
            Aabb::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0))
        }
    }

    pub fn get_contacting_colliders(&self, collider_handle: ColliderHandle) -> Vec<ColliderHandle> {
        self.narrow_phase
            .contacts_with(collider_handle)
            .map(|contact_pair| {
                if contact_pair.collider1 == collider_handle {
                    contact_pair.collider2
                } else {
                    contact_pair.collider1
                }
            })
            .collect()
    }

    pub fn set_gravity(&mut self, gravity: Vector<Real>) {
        self.gravity = gravity;
    }

    pub fn set_linear_dampening(&mut self, body_handle: RigidBodyHandle, dampening: Real) {
        if let Some(rigid_body) = self.bodies.get_mut(body_handle) {
            rigid_body.set_linear_damping(dampening);
        } else {
            error!("Could not set linear dampening cause rigid body didnt exist!");
        }
    }

    pub fn set_bounciness(&mut self, collider_handle: ColliderHandle, coefficient: Real) {
        if let Some(collider) = self.colliders.get_mut(collider_handle) {
            collider.set_restitution(coefficient);
        } else {
            error!("Could not set linear dampening cause rigid body didnt exist!");
        }
    }

    pub fn s_apply_force(&mut self, body_handle: RigidBodyHandle, force: Vector<Real>) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.add_force(force, true);
        } else {
            error!("Could not find body {:?} to apply force to", body_handle);
        }
    }

    pub fn s_set_velocity(&mut self, body_handle: RigidBodyHandle, linvel: Vector<Real>) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.set_linvel(linvel, true);
        } else {
            error!("Could not find body {:?} to apply force to", body_handle);
        }
    }

    pub fn apply_force(
        rigid_body_set: &mut RigidBodySet,
        body_handle: RigidBodyHandle,
        force: Vector<Real>,
    ) {
        if let Some(body) = rigid_body_set.get_mut(body_handle) {
            body.add_force(force, true);
        } else {
            error!("Could not find body {:?} to apply force to", body_handle);
        }
    }

    pub fn apply_impulse(&mut self, body_handle: RigidBodyHandle, impulse: Vector<Real>) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.apply_impulse(impulse, true);
        } else {
            error!("Could not find body {:?} to apply force to", body_handle);
        }
    }

    pub fn move_collider(&mut self, collider_handle: ColliderHandle, movement: Vector<Real>) {
        if let Some(collider) = self.colliders.get_mut(collider_handle) {
            collider.set_translation(Vector::new(
                collider.translation().x + (movement.x / self.simulation_scaling_factor),
                collider.translation().y + (movement.y / self.simulation_scaling_factor),
            ));
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    pub fn s_get_collider_translation(&self, collider_handle: ColliderHandle) -> Vector<Real> {
        if let Some(collider) = self.colliders.get(collider_handle) {
            return collider.translation().clone();
        }

        Vector::zeros()
    }

    pub fn set_translation_for_collider(
        &mut self,
        position: Vector<Real>,
        collider_handle: ColliderHandle,
    ) {
        if let Some(collider) = self.colliders.get_mut(collider_handle) {
            collider.set_translation(position);
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    pub fn set_translation_for_rigid_body(
        &mut self,
        position: Vector<Real>,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.set_translation(position, true);
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    pub fn set_translation_for_rigid_body_x(
        &mut self,
        position_x: Real,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.set_translation(
                Vector::new(
                    position_x / self.simulation_scaling_factor,
                    body.translation().y,
                ),
                true,
            );
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    pub fn set_translation_for_rigid_body_y(
        &mut self,
        position_y: Real,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.set_translation(
                Vector::new(
                    body.translation().x,
                    position_y / self.simulation_scaling_factor,
                ),
                true,
            );
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    pub fn set_collision_group(
        &mut self,
        collider_handler: ColliderHandle,
        group: InteractionGroups,
    ) {
        if let Some(collider) = self.colliders.get_mut(collider_handler) {
            collider.set_collision_groups(group);
        }
    }

    pub fn add_terrain_chunk(
        &mut self,
        chunk: &TerrainGenTerrainChunk,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(Vector::new(
                ((chunk.x * chunk.tile_width) + (chunk.tile_width / 2.0))
                    / self.simulation_scaling_factor,
                ((chunk.y * chunk.tile_height) - (chunk.tile_height / 2.0))
                    / self.simulation_scaling_factor,
            ))
            .build();

        let body_handle = self.bodies.insert(rigid_body);

        let collider_handle = self.colliders.insert_with_parent(
            ColliderBuilder::cuboid(
                ((chunk.tile_width * chunk.tiles_in_x) / 2.0) / self.simulation_scaling_factor,
                ((chunk.tile_height * chunk.tiles_in_y) / 2.0) / self.simulation_scaling_factor,
            )
            .friction(0.5)
            .collision_groups(COL_GROUP_A)
            .build(),
            body_handle,
            &mut self.bodies,
        );

        (body_handle, collider_handle)
    }

    pub fn add_static_collision_area(
        &mut self,
        half_x: Real,
        half_y: Real,
        pos_x: Real,
        pos_y: Real,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(
            half_x / self.simulation_scaling_factor,
            half_y / self.simulation_scaling_factor,
        )
        .translation(Vector::new(
            pos_x / self.simulation_scaling_factor,
            pos_y / self.simulation_scaling_factor,
        ))
        .sensor(true)
        .collision_groups(COL_GROUP_A)
        .active_collision_types(ActiveCollisionTypes::default() | ActiveCollisionTypes::FIXED_FIXED)
        .build();

        self.colliders.insert(collider)
    }

    pub fn add_static_body_cuboid(
        &mut self,
        half_x: Real,
        half_y: Real,
        pos_x: Real,
        pos_y: Real,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let collider = ColliderBuilder::cuboid(
            half_x / self.simulation_scaling_factor,
            half_y / self.simulation_scaling_factor,
        )
        .collision_groups(COL_GROUP_A)
        .build();

        let rigid_body = RigidBodyBuilder::fixed()
            .translation(Vector::new(
                pos_x / self.simulation_scaling_factor,
                pos_y / self.simulation_scaling_factor,
            ))
            .build();

        let body_handle = self.bodies.insert(rigid_body);

        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);

        (body_handle, collider_handle)
    }

    pub fn get_isometry_from_world_coordinates(&self, pos_x: Real, pos_y: Real) -> Isometry<Real> {
        Isometry::new(
            Vector::new(
                pos_x / self.simulation_scaling_factor,
                pos_y / self.simulation_scaling_factor,
            ),
            0.0,
        )
    }

    pub fn add_dynamic_body_cuboid(
        &mut self,
        half_x: Real,
        half_y: Real,
        pos_x: Real,
        pos_y: Real,
        dampening: Real,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let collider = ColliderBuilder::round_cuboid(
            (half_x - 1.0) / self.simulation_scaling_factor,
            (half_y - 1.0) / self.simulation_scaling_factor,
            0.1 / self.simulation_scaling_factor,
        )
        .active_events(ActiveEvents::CONTACT_FORCE_EVENTS)
        .density(2.0)
        .collision_groups(COL_GROUP_A)
        .build();

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(
                pos_x / self.simulation_scaling_factor,
                pos_y / self.simulation_scaling_factor,
            ))
            .linear_damping(dampening)
            .lock_rotations()
            .ccd_enabled(true)
            .build();

        let body_handle = self.bodies.insert(rigid_body);

        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);

        (body_handle, collider_handle)
    }

    pub fn remove_collider(&mut self, collider_handle: ColliderHandle) {
        self.colliders
            .remove(collider_handle, &mut self.islands, &mut self.bodies, false);
    }

    pub fn remove_rigid_body(&mut self, body_handle: RigidBodyHandle) {
        self.bodies.remove(
            body_handle,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    pub fn new() -> RapierSimulation {
        let (contact_send, contact_receiver) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_receiver) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

        RapierSimulation {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vector::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            multibody_joints: MultibodyJointSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            physics_hooks: (),
            simulation_scaling_factor: 100.0,
            events: Box::from(event_handler),
        }
    }
}
