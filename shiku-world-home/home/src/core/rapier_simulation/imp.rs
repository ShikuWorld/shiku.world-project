use std::ops::Deref;

use crate::core::basic_kinematic_character_controller::{
    BasicKinematicCharacterController, CharacterAutostep, CharacterCollision, CharacterLength,
};
use crate::core::blueprint::scene::def::KinematicCharacterControllerProps;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::terrain_gen::TerrainGenTerrainChunk;
use log::{debug, error};
use rapier2d::crossbeam;
use rapier2d::na::{Point2, Vector2};
use rapier2d::prelude::*;

pub const COL_GROUP_A: InteractionGroups = InteractionGroups::new(Group::GROUP_1, Group::GROUP_1);
pub const COL_GROUP_B: InteractionGroups = InteractionGroups::new(Group::GROUP_2, Group::GROUP_2);

impl RapierSimulation {
    pub fn update(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase_multi_sap,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            self.events.deref(),
        );
    }

    pub fn move_character(
        &mut self,
        character_controller: &BasicKinematicCharacterController,
        rigid_body_handle: RigidBodyHandle,
        collider_handle: ColliderHandle,
        desired_translation: Vector<Real>,
        character_collisions: &mut Vec<CharacterCollision>,
    ) -> (bool, bool) {
        let mut rigid_body_position = self
            .bodies
            .get(rigid_body_handle)
            .map(|body| *body.position())
            .unwrap_or_default();
        let mut grounded = false;
        let mut is_sliding_down_slope = false;
        if let Some(collider) = self.colliders.get(collider_handle) {
            let corrected_movement = character_controller.move_shape(
                self.integration_parameters.dt,
                &self.bodies,
                &self.colliders,
                &self.query_pipeline,
                collider.shape(),
                &rigid_body_position,
                desired_translation,
                QueryFilter::default()
                    .exclude_rigid_body(rigid_body_handle)
                    .exclude_sensors(),
                |collision| {
                    character_collisions.push(collision);
                },
            );

            character_controller.solve_character_collision_impulses(
                self.integration_parameters.dt,
                &mut self.bodies,
                &self.colliders,
                &self.query_pipeline,
                collider.shape(),
                5.0,
                character_collisions.iter(),
                QueryFilter::default().exclude_rigid_body(rigid_body_handle),
            );

            if corrected_movement.translation.y.abs() > 0.0001 {
                rigid_body_position.translation.y += corrected_movement.translation.y;
            }
            if corrected_movement.translation.x.abs() > 0.0001 {
                rigid_body_position.translation.x += corrected_movement.translation.x;
            }
            grounded = corrected_movement.grounded;
            is_sliding_down_slope = corrected_movement.is_sliding_down_slope;
        }
        if let Some(rigid_body) = self.bodies.get_mut(rigid_body_handle) {
            rigid_body.set_position(rigid_body_position, true);
        }
        (grounded, is_sliding_down_slope)
    }

    pub fn get_single_character_collision_impulse(
        &mut self,
        character_controller: &BasicKinematicCharacterController,
        collider_handle: &ColliderHandle,
        character_collision: &CharacterCollision,
    ) -> Option<Vector<Real>> {
        character_controller.get_single_character_collision_impulse(
            self.integration_parameters.dt,
            &mut self.bodies,
            &self.colliders,
            collider_handle,
            character_collision,
        )
    }

    pub fn is_collider_handle_part_of_kinematic_body(
        &self,
        collider_handle: &ColliderHandle,
    ) -> bool {
        self.colliders
            .get(*collider_handle)
            .and_then(|collider| collider.parent())
            .and_then(|body_handle| self.bodies.get(body_handle))
            .map(|body| body.is_kinematic())
            .unwrap_or(false)
    }

    pub fn create_kinematic_character_controller(
        props: &KinematicCharacterControllerProps,
    ) -> BasicKinematicCharacterController {
        let mut kinematic = BasicKinematicCharacterController::default();
        if let Some(auto_step_props) = &props.autostep {
            kinematic.autostep = Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(auto_step_props.max_height),
                min_width: CharacterLength::Absolute(auto_step_props.min_width),
                include_dynamic_bodies: auto_step_props.include_dynamic_bodies,
            });
        } else {
            kinematic.autostep = None;
        }
        kinematic.offset = CharacterLength::Absolute(props.offset.abs());
        kinematic.slide = props.slide;
        kinematic.up = UnitVector::new_normalize(Vector2::new(props.up.0, props.up.1));
        kinematic.normal_nudge_factor = props.normal_nudge_factor.abs();
        kinematic.max_slope_climb_angle = props.max_slope_climb_angle;
        kinematic.min_slope_slide_angle = props.min_slope_slide_angle;
        if let Some(snap_to_ground) = props.snap_to_ground {
            kinematic.snap_to_ground = Some(CharacterLength::Absolute(snap_to_ground));
        } else {
            kinematic.snap_to_ground = None;
        }
        kinematic
    }

    pub fn get_intersecting_colliders(
        &self,
        collider_handle: ColliderHandle,
    ) -> Vec<ColliderHandle> {
        self.narrow_phase
            .intersection_pairs_with(collider_handle)
            .filter(|(_, _, intersecting)| *intersecting)
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

    pub fn add_polyine(
        &mut self,
        vertices: Vec<Point2<Real>>,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let body_handle = self.add_fixed_rigid_body(0.0, 0.0);
        let collider = ColliderBuilder::polyline(vertices, None)
            .active_events(ActiveEvents::all())
            .build();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);

        (body_handle, collider_handle)
    }

    pub fn get_contacting_colliders(&self, collider_handle: ColliderHandle) -> Vec<ColliderHandle> {
        self.narrow_phase
            .contact_pairs_with(collider_handle)
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

    pub fn add_force(
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
                collider.translation().x + (movement.x),
                collider.translation().y + (movement.y),
            ));
        } else {
            //TODO: Log critical errors like these only once somehow
            error!("Body handle to update did not exist, this should never happen!")
        }
    }

    fn atan2_approx(im: f32, re: f32) -> f32 {
        let abs_y = im.abs() + 1e-10; // kludge to prevent 0/0 condition
        let r = (re - re.signum() * abs_y) / (abs_y + re.abs());
        let mut angle = std::f32::consts::PI / 2.0 - (std::f32::consts::PI / 4.0) * re.signum();
        angle += (0.1963 * r * r - 0.9817) * r;
        angle * im.signum()
    }

    pub fn get_rigid_body_translation(
        &self,
        rigid_body_handle: RigidBodyHandle,
    ) -> (Real, Real, Real) {
        if let Some(rigid_body) = self.bodies.get(rigid_body_handle) {
            let position = rigid_body.position();
            return (
                position.translation.x,
                position.translation.y,
                position.rotation.angle(), //Self::atan2_approx(position.rotation.im, position.rotation.re),
            );
        }

        (0.0, 0.0, 0.0)
    }

    pub fn s_get_collider_translation(&self, collider_handle: ColliderHandle) -> Vector<Real> {
        if let Some(collider) = self.colliders.get(collider_handle) {
            return *collider.translation();
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

    pub fn set_translation_and_rotation_for_rigid_body(
        &mut self,
        position: Vector<Real>,
        rotation: Real,
        body_handle: RigidBodyHandle,
    ) {
        if let Some(body) = self.bodies.get_mut(body_handle) {
            body.set_translation(position, true);
            body.set_rotation(Rotation::new(rotation), true);
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
            body.set_translation(Vector::new(position_x, body.translation().y), true);
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
            body.set_translation(Vector::new(body.translation().x, position_y), true);
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
                (chunk.x * chunk.tile_width) + (chunk.tile_width / 2.0),
                (chunk.y * chunk.tile_height) - (chunk.tile_height / 2.0),
            ))
            .build();

        let body_handle = self.bodies.insert(rigid_body);

        let collider_handle = self.colliders.insert_with_parent(
            ColliderBuilder::cuboid(
                (chunk.tile_width * chunk.tiles_in_x) / 2.0,
                (chunk.tile_height * chunk.tiles_in_y) / 2.0,
            )
            .friction(0.5)
            .collision_groups(COL_GROUP_A)
            .build(),
            body_handle,
            &mut self.bodies,
        );

        (body_handle, collider_handle)
    }

    pub fn create_cuboid_collider(
        &mut self,
        half_x: Real,
        half_y: Real,
        body_handle: RigidBodyHandle,
        is_sensor: bool,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(half_x, half_y)
            .sensor(is_sensor)
            .active_events(ActiveEvents::all())
            .build();

        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies)
    }

    pub fn create_ball_collider(
        &mut self,
        radius: Real,
        body_handle: RigidBodyHandle,
        is_sensor: bool,
    ) -> ColliderHandle {
        debug!("Creating ball collider with is_sensor: {}", is_sensor);
        let collider = ColliderBuilder::ball(radius)
            .active_events(ActiveEvents::all())
            .active_collision_types(ActiveCollisionTypes::all())
            .sensor(is_sensor)
            .build();

        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies)
    }

    pub fn create_capsule_x_collider(
        &mut self,
        half_y: Real,
        radius: Real,
        body_handle: RigidBodyHandle,
        is_sensor: bool,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::capsule_x(half_y, radius)
            .sensor(is_sensor)
            .active_events(ActiveEvents::all())
            .build();

        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies)
    }

    pub fn create_capsule_y_collider(
        &mut self,
        half_x: Real,
        radius: Real,
        body_handle: RigidBodyHandle,
        is_sensor: bool,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::capsule_y(half_x, radius)
            .sensor(is_sensor)
            .active_events(ActiveEvents::all())
            .build();

        self.colliders
            .insert_with_parent(collider, body_handle, &mut self.bodies)
    }

    pub fn get_isometry_from_world_coordinates(&self, pos_x: Real, pos_y: Real) -> Isometry<Real> {
        Isometry::new(Vector::new(pos_x, pos_y), 0.0)
    }

    pub fn add_dynamic_rigid_body(&mut self, pos_x: Real, pos_y: Real) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(pos_x, pos_y))
            .ccd_enabled(true)
            .can_sleep(false)
            .linear_damping(0.5)
            .build();

        self.bodies.insert(rigid_body)
    }

    pub fn add_fixed_rigid_body(&mut self, pos_x: Real, pos_y: Real) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::fixed()
            .translation(Vector::new(pos_x, pos_y))
            .build();

        self.bodies.insert(rigid_body)
    }

    pub fn add_kinematic_position_based_rigid_body(
        &mut self,
        pos_x: Real,
        pos_y: Real,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(Vector::new(pos_x, pos_y))
            .build();

        self.bodies.insert(rigid_body)
    }

    pub fn add_kinematic_velocity_based_rigid_body(
        &mut self,
        pos_x: Real,
        pos_y: Real,
    ) -> RigidBodyHandle {
        let rigid_body = RigidBodyBuilder::kinematic_velocity_based()
            .translation(Vector::new(pos_x, pos_y))
            .build();

        self.bodies.insert(rigid_body)
    }

    pub fn remove_collider(&mut self, collider_handle: ColliderHandle) {
        self.colliders
            .remove(collider_handle, &mut self.islands, &mut self.bodies, true);
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

    pub fn add_static_body_cuboid(
        &mut self,
        _half_x: Real,
        _half_y: Real,
        _pos_x: Real,
        _pos_y: Real,
    ) -> (RigidBodyHandle, ColliderHandle) {
        //TODO: Remove
        todo!()
    }

    pub fn add_dynamic_body_cuboid(
        &mut self,
        _half_x: Real,
        _half_y: Real,
        _pos_x: Real,
        _pos_y: Real,
        _damb: Real,
    ) -> (RigidBodyHandle, ColliderHandle) {
        //TODO: Remove
        todo!()
    }

    pub fn add_static_collision_area(
        &mut self,
        _half_x: Real,
        _half_y: Real,
        _pos_x: Real,
        _pos_y: Real,
    ) -> ColliderHandle {
        //TODO: Remove
        todo!()
    }

    pub fn new() -> RapierSimulation {
        let (contact_send, contact_receiver) = crossbeam::channel::unbounded();
        let (intersection_send, intersection_receiver) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(intersection_send, contact_send);

        RapierSimulation {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: Vector::new(0.0, 1.0),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase_multi_sap: BroadPhaseMultiSap::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            multibody_joints: MultibodyJointSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            physics_hooks: (),
            events: Box::from(event_handler),
            query_pipeline: QueryPipeline::new(),
            contact_receiver,
            intersection_receiver,
        }
    }
}
