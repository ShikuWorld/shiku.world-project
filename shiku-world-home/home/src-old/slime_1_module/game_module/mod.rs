use std::collections::{HashMap, HashSet};
use std::time::Instant;

use log::{error, warn};
use rapier2d::na::Vector2;
use rapier2d::parry::bounding_volume::BoundingVolume;
use rapier2d::prelude::{Real, RigidBodyHandle, Vector};

use crate::core::blending_mode;
use crate::core::entity::def::EntityId;
use crate::core::entity::physics::Physical;
use crate::core::entity::physics::PhysicalShape;
use crate::core::entity::render::{CameraSettings, ShowEffect, SimpleImageEffect};
use crate::core::entity_manager::{ColliderEntityMap, EntityManager};
use crate::core::game_module_communication::GameModuleCommunicationCallbacks;
use crate::core::guest::{ModuleEnterSlot, ModuleExitSlot, ProviderUserId};
use crate::core::module::{
    GameSystemToGuest, GameSystemToGuestEvent, GuestEvent, GuestInput, GuestStateChange,
    ModuleOutputSender, ModuleToSystem, ModuleToSystemEvent, MouseInputSchema,
};
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::rapier_simulation::imp::{COL_GROUP_A, COL_GROUP_B};
use crate::core::tween::TweenProp;
use crate::core::TARGET_FRAME_DURATION;
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::GuestId;
use crate::resource_module::map::def::{GeneralObject, LayerName};
use crate::slime_1_module::def::{Slime1BasicGameModule, Slime1Module, Slime1SimulationConfig};
use crate::slime_1_module::game_module::cave_opening::CaveOpening;
use crate::slime_1_module::game_module::generated::{
    DoorEntity, Guest, GuestDead, GuestEntity, GuestNameplate, HeartSmoll, HeartWallGlow, Observer,
    OpeningAreaPlate, OpeningAreaPlateEntity, Slime1GameEntityManager, Slime1GameObject,
    SlimeCharge, SlimeLight, WallPlatform, WallPlatformOpener,
};
use crate::slime_1_module::game_module::guest_movement::{
    GuestDirections, GuestMovement, GuestMovementState,
};

pub mod cave_opening;
pub mod generated;
pub mod guest_movement;

pub struct Slime1GuestEntity {
    pub id: EntityId,
    pub dead_entity: Option<EntityId>,
    pub name_plate_id: EntityId,
    pub light_id: Option<EntityId>,
    pub heart_id: Option<EntityId>,
    pub smoll_heart_id: Option<EntityId>,
    pub kind: Slime1GameObject,
}

pub struct Slime1Guest {
    pub guest_id: GuestId,
    pub guest_name: String,
    pub entity: Slime1GuestEntity,
    pub guest_input: GuestInput,
    pub found_secret_map: HashSet<String>,
    pub time_of_last_guest_input: Instant,
    pub light_touched_time: Option<Instant>,
    pub grabbed_by_light: bool,
    pub squished_timer: Real,
    pub guest_movement: GuestMovement,
    pub in_exit_slot: Option<ModuleExitSlot>,
}

pub struct Slime1GameModule {
    slime_1_guest_map: HashMap<GuestId, Slime1Guest>,
    to_guest_events: Vec<GameSystemToGuest>,
    logged_out_guest_entity_map: HashMap<ProviderUserId, (Slime1GuestEntity, GuestMovement)>,
    guests_in_the_grabs: Vec<RigidBodyHandle>,
    outstanding_guest_teleportations: Vec<(RigidBodyHandle, Vector2<Real>)>,
    cave_opening: CaveOpening,
}

impl Slime1GameModule {
    pub fn new(basic_game_module: &mut Slime1BasicGameModule) -> Slime1GameModule {
        Slime1GameModule {
            slime_1_guest_map: HashMap::new(),
            to_guest_events: Vec::new(),
            logged_out_guest_entity_map: HashMap::new(),
            guests_in_the_grabs: Vec::new(),
            outstanding_guest_teleportations: Vec::new(),
            cave_opening: CaveOpening::new(basic_game_module),
        }
    }

    /*pub fn set_simulation_config(
        basic_game_module: &mut Slime1BasicGameModule,
        simulation_config: Slime1SimulationConfig,
    ) {
        basic_game_module.simulation_config = simulation_config;

        for entity in basic_game_module.game_entity_manager.guest_map.values() {
            basic_game_module.simulation.set_linear_dampening(
                entity.physics.body_handle,
                basic_game_module.simulation_config.guest_linear_dampening,
            );
            basic_game_module.simulation.set_bounciness(
                entity.physics.collider_handle,
                basic_game_module.simulation_config.guest_bounciness,
            );
        }
    }*/

    fn are_all_doors_open(
        opening_area_plate: &OpeningAreaPlateEntity,
        door_map: &HashMap<EntityId, DoorEntity>,
    ) -> bool {
        let mut all_open = true;
        all_open =
            all_open && Self::is_door_open(&opening_area_plate.game_state.door_to_open_1, door_map);
        all_open =
            all_open && Self::is_door_open(&opening_area_plate.game_state.door_to_open_2, door_map);
        all_open =
            all_open && Self::is_door_open(&opening_area_plate.game_state.door_to_open_3, door_map);
        all_open =
            all_open && Self::is_door_open(&opening_area_plate.game_state.door_to_open_4, door_map);
        all_open
    }

    fn is_door_open(door_id: &EntityId, door_map: &HashMap<EntityId, DoorEntity>) -> bool {
        if door_id == "NOT_FOUND" {
            return true;
        }

        if let Some(door) = door_map.get(door_id) {
            door.game_state.is_open
        } else {
            false
        }
    }

    fn update_death_by_spikes(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        let mut game_entity_manager = &mut basic_game_module.game_entity_manager;
        let mut simulation = &mut basic_game_module.simulation;
        let mut death_prop_map = &mut game_entity_manager.death_prop_map;
        let mut guest_map = &mut game_entity_manager.guest_map;
        let mut is_guest_dead = false;
        let collider_entity_map = &game_entity_manager.collider_entity_map;

        if let Some(guest_entity) = guest_map.get_mut(&slime_1_guest.entity.id) {
            collider_entity_map.entities_from_colliders(
                &simulation.get_contacting_colliders(guest_entity.physics.collider_handle),
                &mut death_prop_map,
                |death_prop| {
                    is_guest_dead = true;
                },
            );
        }
        if is_guest_dead {
            Self::kill_guest(
                slime_1_guest,
                game_entity_manager,
                simulation,
                module_output_sender,
            );
        }
    }

    fn kill_guest(
        slime_1_guest: &mut Slime1Guest,
        game_entity_manager: &mut Slime1GameEntityManager,
        simulation: &mut RapierSimulation,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        game_entity_manager.remove_guest_nameplate(&slime_1_guest.entity.name_plate_id, simulation);
        if let Some(smoll_heart_id) = &slime_1_guest.entity.smoll_heart_id {
            game_entity_manager.remove_heart_smoll(smoll_heart_id, simulation);
        }
        if let Some(guest_entity) =
            game_entity_manager.remove_guest(&slime_1_guest.entity.id, simulation)
        {
            let slime_dead_variant =
                &GuestDead::VARIANTS.get_variant(&slime_1_guest.guest_movement.skin_name);
            let dead_entity_id = game_entity_manager.create_guest_dead(
                GuestDead {
                    flame: "".to_string(),
                    heart_color: guest_entity.game_state.heart_color.clone(),
                    slime: "".to_string(),
                },
                Vector::new(
                    guest_entity.isometry.translation.x * simulation.simulation_scaling_factor,
                    guest_entity.isometry.translation.y * simulation.simulation_scaling_factor,
                ),
                slime_dead_variant,
                slime_dead_variant.gid_idle.to_string(),
                simulation,
                |guest_dead_entity| {
                    guest_dead_entity.render.layer = LayerName::FG3;
                },
            );

            game_entity_manager.set_camera_entity_for_guest(
                slime_1_guest.guest_id.clone(),
                dead_entity_id.clone(),
            );

            if let Err(err) = module_output_sender
                .game_system_to_guest_sender
                .send(GuestEvent {
                    guest_id: slime_1_guest.guest_id.clone(),
                    event_type: GameSystemToGuestEvent::SetCamera(
                        dead_entity_id.clone(),
                        Slime1Module::module_name(),
                        CameraSettings::default(),
                    ),
                })
            {
                error!("Could not set camera after guest died, wat!");
            }

            slime_1_guest.entity.dead_entity = Some(dead_entity_id);
        }
    }

    fn check_guest_and_observer_exit(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        output_sender: &mut ModuleOutputSender,
    ) {
        if slime_1_guest.in_exit_slot.is_some() {
            return;
        }

        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get(&slime_1_guest.entity.id)
        {
            for (collider1, collider2, intersecting) in basic_game_module
                .simulation
                .narrow_phase
                .intersections_with(guest_entity.physics.collider_handle)
            {
                if !intersecting {
                    continue;
                }

                let other_collider_handle = if collider1 == guest_entity.physics.collider_handle {
                    collider2
                } else {
                    collider1
                };

                if let Some((exit_entity_id, Slime1GameObject::ExitArea)) = basic_game_module
                    .game_entity_manager
                    .collider_entity_map
                    .get(&other_collider_handle)
                {
                    if let Some(exit_area) = basic_game_module
                        .game_entity_manager
                        .exit_area_map
                        .get(exit_entity_id)
                    {
                        slime_1_guest.in_exit_slot = Some(exit_area.game_state.slot_id.clone());
                    }
                };
            }
        }

        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .observer_map
            .get(&slime_1_guest.entity.id)
        {
            for (collider1, collider2, intersecting) in basic_game_module
                .simulation
                .narrow_phase
                .intersections_with(guest_entity.physics.collider_handle)
            {
                if !intersecting {
                    continue;
                }

                let other_collider_handle = if collider1 == guest_entity.physics.collider_handle {
                    collider2
                } else {
                    collider1
                };

                if let Some((exit_entity_id, Slime1GameObject::ExitArea)) = basic_game_module
                    .game_entity_manager
                    .collider_entity_map
                    .get(&other_collider_handle)
                {
                    if let Some(exit_area) = basic_game_module
                        .game_entity_manager
                        .exit_area_map
                        .get(exit_entity_id)
                    {
                        slime_1_guest.in_exit_slot = Some(exit_area.game_state.slot_id.clone());
                    }
                };
            }
        }

        if let Some(module_exit_slot) = &slime_1_guest.in_exit_slot {
            if let Err(err) = output_sender.module_to_system_sender.send(GuestEvent {
                guest_id: slime_1_guest.guest_id.clone(),
                event_type: ModuleToSystemEvent::GuestStateChange(GuestStateChange::ExitModule(
                    module_exit_slot.clone(),
                )),
            }) {
                error!(
                    "Could not send exit module event, this is very bad! {:?}",
                    err
                );
            }
        }
    }

    fn update_wall_opening_areas(basic_game_module: &mut Slime1BasicGameModule) {
        let simulation = &mut basic_game_module.simulation;
        let game_entity_manager = &mut basic_game_module.game_entity_manager;
        let collider_entity_map = &game_entity_manager.collider_entity_map;
        let wall_platform_opener_map = &mut game_entity_manager.wall_platform_opener_map;

        for wall_platform_opener in wall_platform_opener_map.values_mut() {
            let mut guest_in_area = false;
            let mut heart_colors_match = false;

            collider_entity_map.entities_from_colliders(
                &simulation
                    .get_intersecting_colliders(wall_platform_opener.physics.collider_handle),
                &mut game_entity_manager.guest_map,
                |guest_entity| {
                    let area_translation = simulation
                        .s_get_collider_translation(wall_platform_opener.physics.collider_handle);

                    let distance_vec = Vector::new(
                        guest_entity.isometry.translation.x - area_translation.x,
                        guest_entity.isometry.translation.y - area_translation.y,
                    );

                    let distance = distance_vec.magnitude();
                    if distance < 0.05 {
                        guest_in_area = true;
                    }

                    heart_colors_match = heart_colors_match
                        || wall_platform_opener.game_state.heart_color.len() == 0
                        || (guest_entity.game_state.heart_color
                            == wall_platform_opener.game_state.heart_color);

                    let mut force_scale = if distance > 0.13 {
                        0.0
                    } else {
                        (1.0 - (distance / 0.13)) * -1.6
                    };

                    simulation.s_apply_force(
                        guest_entity.physics.body_handle,
                        distance_vec.scale(force_scale),
                    );
                },
            );

            if guest_in_area && heart_colors_match {
                wall_platform_opener.set_graphic_id(
                    WallPlatformOpener::VARIANTS
                        .get_variant(&wall_platform_opener.game_state.opener_variant)
                        .gid_on,
                );
                wall_platform_opener.game_state.active = true;
            } else {
                wall_platform_opener.set_graphic_id(
                    WallPlatformOpener::VARIANTS
                        .get_variant(&wall_platform_opener.game_state.opener_variant)
                        .gid_off,
                );
                wall_platform_opener.game_state.active = false;
            }
        }
    }

    fn update_opening_areas(basic_game_module: &mut Slime1BasicGameModule) {
        let simulation = &mut basic_game_module.simulation;
        let game_entity_manager = &mut basic_game_module.game_entity_manager;
        let collider_entity_map = &game_entity_manager.collider_entity_map;
        let opening_area_plate_map = &mut game_entity_manager.opening_area_plate_map;

        for opening_area_plate in opening_area_plate_map.values_mut() {
            let mut guests_in_area = 0;

            collider_entity_map.entities_from_colliders(
                &simulation.get_intersecting_colliders(
                    opening_area_plate.physics.activation.collider_handle,
                ),
                &mut game_entity_manager.guest_map,
                |guest_entity| {
                    let area_translation = simulation.s_get_collider_translation(
                        opening_area_plate.physics.activation.collider_handle,
                    );

                    let distance_vec = Vector::new(
                        guest_entity.isometry.translation.x - area_translation.x,
                        guest_entity.isometry.translation.y - area_translation.y,
                    );

                    let distance = distance_vec.magnitude();
                    if distance < 0.05 {
                        guests_in_area += 1;
                    }

                    if opening_area_plate.render.graphic_id
                        != OpeningAreaPlate::VARIANTS
                            .get_variant(&opening_area_plate.game_state.opener_variant)
                            .gid_done
                    {
                        let mut force_scale = if distance > 0.13 {
                            0.0
                        } else {
                            (1.0 - (distance / 0.13)) * -1.6
                        };

                        simulation.s_apply_force(
                            guest_entity.physics.body_handle,
                            distance_vec.scale(force_scale),
                        );
                    }
                },
            );

            if Self::are_all_doors_open(opening_area_plate, &game_entity_manager.door_map) {
                opening_area_plate.set_graphic_id(
                    OpeningAreaPlate::VARIANTS
                        .get_variant(&opening_area_plate.game_state.opener_variant)
                        .gid_done,
                );
            } else if guests_in_area == 0 {
                opening_area_plate.set_graphic_id(
                    OpeningAreaPlate::VARIANTS
                        .get_variant(&opening_area_plate.game_state.opener_variant)
                        .gid_default,
                );
            } else {
                opening_area_plate.set_graphic_id(
                    OpeningAreaPlate::VARIANTS
                        .get_variant(&opening_area_plate.game_state.opener_variant)
                        .gid_activated,
                );
            }
        }
    }

    fn update_teleporter(
        basic_game_module: &mut Slime1BasicGameModule,
        outstanding_guest_teleportations: &mut Vec<(RigidBodyHandle, Vector2<Real>)>,
    ) {
        let mut teleport_observer = None;
        for teleporter_start in basic_game_module
            .game_entity_manager
            .teleport_start_map
            .values_mut()
        {
            if let Some(observer_entity) = basic_game_module
                .game_entity_manager
                .observer_map
                .values()
                .next()
            {
                let distance = ((teleporter_start.isometry.translation.y
                    - observer_entity.isometry.translation.y)
                    .powf(2.0)
                    + (teleporter_start.isometry.translation.x
                        - observer_entity.isometry.translation.x)
                        .powf(2.0))
                .sqrt();

                if distance < 0.1 {
                    if let Some(teleport_end) = basic_game_module
                        .game_entity_manager
                        .teleport_end_map
                        .get(&teleporter_start.game_state.teleport_end)
                    {
                        if let Some(teleport_end_collider) = basic_game_module
                            .simulation
                            .colliders
                            .get(teleport_end.physics.collider_handle)
                        {
                            teleport_observer = Some((
                                observer_entity.physics.collider_handle,
                                *teleport_end_collider.translation(),
                            ));
                        }
                    }
                }
            }

            for (collider1_handle, collider2_handle, intersecting) in basic_game_module
                .simulation
                .narrow_phase
                .intersections_with(teleporter_start.physics.collider_handle)
            {
                if !intersecting {
                    continue;
                }

                let other_collider_handle =
                    if collider1_handle == teleporter_start.physics.collider_handle {
                        collider2_handle
                    } else {
                        collider1_handle
                    };

                if let Some((guest_entity_id, Slime1GameObject::Guest)) = basic_game_module
                    .game_entity_manager
                    .collider_entity_map
                    .get(&other_collider_handle)
                {
                    if let (Some(teleport_end), Some(guest_entity)) = (
                        basic_game_module
                            .game_entity_manager
                            .teleport_end_map
                            .get(&teleporter_start.game_state.teleport_end),
                        basic_game_module
                            .game_entity_manager
                            .guest_map
                            .get(guest_entity_id),
                    ) {
                        if let Some(teleport_end_collider) = basic_game_module
                            .simulation
                            .colliders
                            .get(teleport_end.physics.collider_handle)
                        {
                            outstanding_guest_teleportations.push((
                                guest_entity.physics.body_handle,
                                *teleport_end_collider.translation(),
                            ));
                        }
                    }
                }
            }
        }

        if let Some((collider_handle, position)) = teleport_observer {
            basic_game_module
                .simulation
                .set_translation_for_collider(position, collider_handle);
        }

        for (rigid_body_handle, position) in outstanding_guest_teleportations.drain(..) {
            basic_game_module
                .simulation
                .set_translation_for_rigid_body(position, rigid_body_handle);
        }
    }

    fn update_wall_platforms(basic_game_module: &mut Slime1BasicGameModule) {
        let simulation = &mut basic_game_module.simulation;
        for wall_platform in basic_game_module
            .game_entity_manager
            .wall_platform_map
            .values_mut()
        {
            let mut assigned_openers_count = 0;
            let mut assigned_openers_open_count = 0;
            let mut guest_inside_wall = false;

            basic_game_module
                .game_entity_manager
                .collider_entity_map
                .entities_from_colliders(
                    &simulation
                        .get_intersecting_colliders(wall_platform.physics.sensor.collider_handle),
                    &mut basic_game_module.game_entity_manager.guest_map,
                    |guest| {
                        guest_inside_wall = true;
                    },
                );

            for wall_platform_opener in basic_game_module
                .game_entity_manager
                .wall_platform_opener_map
                .get(&wall_platform.game_state.opener)
            {
                assigned_openers_count += 1;
                if wall_platform_opener.game_state.active {
                    assigned_openers_open_count += 1;
                }
            }

            for wall_platform_opener in basic_game_module
                .game_entity_manager
                .wall_platform_opener_map
                .get(&wall_platform.game_state.opener_2)
            {
                assigned_openers_count += 1;
                if wall_platform_opener.game_state.active {
                    assigned_openers_open_count += 1;
                }
            }

            for wall_platform_opener in basic_game_module
                .game_entity_manager
                .wall_platform_opener_map
                .get(&wall_platform.game_state.opener_3)
            {
                assigned_openers_count += 1;
                if wall_platform_opener.game_state.active {
                    assigned_openers_open_count += 1;
                }
            }

            if assigned_openers_count == 0 || (wall_platform.game_state.off && guest_inside_wall) {
                continue;
            }

            let variant =
                WallPlatform::VARIANTS.get_variant(&wall_platform.game_state.wall_variant);

            if assigned_openers_open_count == assigned_openers_count {
                wall_platform.game_state.off = true;
                simulation.set_collision_group(
                    wall_platform.physics.platform.collider_handle,
                    COL_GROUP_B,
                );
                wall_platform.set_graphic_id(variant.gid_off);
            } else {
                wall_platform.game_state.off = false;
                simulation.set_collision_group(
                    wall_platform.physics.platform.collider_handle,
                    COL_GROUP_A,
                );
                wall_platform.set_graphic_id(variant.gid_on);
            }
        }
    }

    fn update_doors(basic_game_module: &mut Slime1BasicGameModule) {
        for door in basic_game_module.game_entity_manager.door_map.values_mut() {
            let mut assigned_opening_areas = 0;
            let mut assigned_opening_areas_open = 0;
            for opening_area_plate in basic_game_module
                .game_entity_manager
                .opening_area_plate_map
                .values()
            {
                if door.id == opening_area_plate.game_state.door_to_open_1
                    || door.id == opening_area_plate.game_state.door_to_open_2
                    || door.id == opening_area_plate.game_state.door_to_open_3
                    || door.id == opening_area_plate.game_state.door_to_open_4
                {
                    assigned_opening_areas += 1;
                    if opening_area_plate.render.graphic_id
                        == OpeningAreaPlate::VARIANTS
                            .get_variant(&opening_area_plate.game_state.opener_variant)
                            .gid_activated
                    {
                        assigned_opening_areas_open += 1;
                    }
                }
            }

            if assigned_opening_areas == 0 {
                continue;
            }

            if assigned_opening_areas_open == assigned_opening_areas
                && !door.game_state.open_change_position.is_running()
            {
                door.game_state.open_change_position.start();
                door.game_state.is_open = true;
            }

            if door.game_state.open_change_position.is_running() {
                door.game_state
                    .open_change_position
                    .update(TARGET_FRAME_DURATION);

                match door.game_state.open_change_position.property {
                    TweenProp::PositionY => basic_game_module
                        .simulation
                        .set_translation_for_rigid_body_y(
                            door.game_state.open_change_position.current_value(),
                            door.physics.body_handle,
                        ),
                    TweenProp::PositionX => basic_game_module
                        .simulation
                        .set_translation_for_rigid_body_x(
                            door.game_state.open_change_position.current_value(),
                            door.physics.body_handle,
                        ),
                }
            }
        }
    }

    fn update_observer(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        guests_in_the_grabs: &mut Vec<RigidBodyHandle>,
    ) {
        if let Some(observer_entity) = basic_game_module
            .game_entity_manager
            .observer_map
            .get_mut(&slime_1_guest.entity.id)
        {
            if slime_1_guest.guest_input.left {
                basic_game_module.simulation.move_collider(
                    observer_entity.physics.collider_handle,
                    Vector::new(-3.0, 0.0),
                );
            }

            if slime_1_guest.guest_input.right {
                basic_game_module.simulation.move_collider(
                    observer_entity.physics.collider_handle,
                    Vector::new(3.0, 0.0),
                );
            }

            if slime_1_guest.guest_input.up {
                basic_game_module.simulation.move_collider(
                    observer_entity.physics.collider_handle,
                    Vector::new(0.0, -3.0),
                );
            }

            if slime_1_guest.guest_input.down {
                basic_game_module.simulation.move_collider(
                    observer_entity.physics.collider_handle,
                    Vector::new(0.0, 3.0),
                );
            }

            if slime_1_guest.guest_input.jump {
                for (collider1, collider2, intersecting) in basic_game_module
                    .simulation
                    .narrow_phase
                    .intersections_with(observer_entity.physics.collider_handle)
                {
                    if !intersecting {
                        continue;
                    }

                    let other_collider_handle =
                        if collider1 == observer_entity.physics.collider_handle {
                            collider2
                        } else {
                            collider1
                        };

                    if let Some((_id, Slime1GameObject::Guest)) = basic_game_module
                        .game_entity_manager
                        .collider_entity_map
                        .get(&other_collider_handle)
                    {
                        if let Some(collider) = basic_game_module
                            .simulation
                            .colliders
                            .get(other_collider_handle)
                        {
                            if let Some(body_handle) = collider.parent() {
                                guests_in_the_grabs.push(body_handle);
                            }
                        }
                    }
                }

                for body_handle in guests_in_the_grabs.drain(..) {
                    basic_game_module.simulation.set_translation_for_rigid_body(
                        Vector::new(
                            observer_entity.isometry.translation.x,
                            observer_entity.isometry.translation.y,
                        ),
                        body_handle,
                    );
                }
            }
        }
    }

    fn update_logged_out_guests(
        entity: &mut Slime1GuestEntity,
        guest_movement: &mut GuestMovement,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        if guest_movement.state != GuestMovementState::Idle {
            guest_movement.afk();
        }

        guest_movement.update(TARGET_FRAME_DURATION);
        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get_mut(&entity.id)
        {
            guest_entity.set_graphic_id(guest_movement.get_current_gid());
        }
    }

    fn update_guest(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        if slime_1_guest.light_touched_time.is_some() && slime_1_guest.entity.light_id.is_none() {
            slime_1_guest.entity.light_id =
                Some(basic_game_module.game_entity_manager.create_slime_light(
                    SlimeLight {},
                    Vector::new(0.0, 0.0),
                    &SlimeLight::VARIANTS.default,
                    SlimeLight::VARIANTS.default.gid_default.to_string(),
                    &mut basic_game_module.simulation,
                    |entity| {
                        entity.parent_entity = Some(slime_1_guest.entity.id.clone());
                        entity.render.offset_2d = (0.0, 0.0);
                        entity.render.layer = LayerName::Guest;
                        entity.render.blending_mode = Some(blending_mode::ADD);
                    },
                ));
        }

        if let (Some(touched_time), Some(light_id)) = (
            &slime_1_guest.light_touched_time,
            &slime_1_guest.entity.light_id,
        ) {
            if Instant::now().duration_since(*touched_time).as_secs() >= 30 {
                basic_game_module
                    .game_entity_manager
                    .remove_slime_light(light_id, &mut basic_game_module.simulation);
                slime_1_guest.light_touched_time = None;
                slime_1_guest.entity.light_id = None;
            }
        }

        Self::update_getting_squished(slime_1_guest, basic_game_module, module_output_sender);
        Self::update_guest_dead(slime_1_guest, basic_game_module);
        Self::update_guest_revival(slime_1_guest, basic_game_module, module_output_sender);
        Self::update_slime_taking_dead_slimes(slime_1_guest, basic_game_module);
        Self::update_death_by_spikes(slime_1_guest, basic_game_module, module_output_sender);
        Self::update_secrets(slime_1_guest, basic_game_module, module_output_sender);
        Self::update_heart_touching(slime_1_guest, basic_game_module);
        Self::update_guest_movement(slime_1_guest, basic_game_module);
    }

    pub fn create_charge_effect(slime_entity_id: String) -> ShowEffect {
        ShowEffect::SimpleImageEffect(SimpleImageEffect {
            graphic_id: SlimeCharge::VARIANTS.default.gid_default.to_string(),
            initial_isometrics_2d: (0.0, 0.0, 0.0),
            layer: LayerName::FG1,
            transparency: 0.5,
            blending_mode: Some(blending_mode::ADD),
            parent_entity: Some(slime_entity_id),
        })
    }

    fn update_heart_wall(
        cave_opening: &mut CaveOpening,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        let mut red_heart_enabled = false;
        let mut yellow_heart_enabled = false;
        let mut blue_heart_enabled = false;
        let mut door_to_remove_option = None;

        for cave_heart_wall in basic_game_module
            .game_entity_manager
            .cave_heart_wall_map
            .values_mut()
        {
            for (collider1, collider2, intersecting) in basic_game_module
                .simulation
                .narrow_phase
                .intersections_with(cave_heart_wall.physics.collider_handle)
            {
                if !intersecting {
                    continue;
                }

                let other_collider_handle = if collider1 == cave_heart_wall.physics.collider_handle
                {
                    collider2
                } else {
                    collider1
                };

                if let Some((id, Slime1GameObject::Guest)) = basic_game_module
                    .game_entity_manager
                    .collider_entity_map
                    .get(&other_collider_handle)
                {
                    if let Some(guest_entity) =
                        basic_game_module.game_entity_manager.guest_map.get(id)
                    {
                        match guest_entity.game_state.heart_color.as_str() {
                            "red" => {
                                red_heart_enabled = true;
                            }
                            "blue" => {
                                blue_heart_enabled = true;
                            }
                            "yellow" => {
                                yellow_heart_enabled = true;
                            }
                            _ => {
                                warn!("No known heart color?");
                            }
                        }
                    }
                }
            }

            if yellow_heart_enabled
                && red_heart_enabled
                && blue_heart_enabled
                && !cave_heart_wall.game_state.cave_open
            {
                cave_heart_wall.game_state.cave_open = true;
                door_to_remove_option = Some(cave_heart_wall.game_state.door_to_remove.clone());
            }

            if let Some(heart_glow) = basic_game_module
                .game_entity_manager
                .heart_wall_glow_map
                .get_mut(&cave_heart_wall.game_state.red_heart)
            {
                if !cave_opening.is_done() && (red_heart_enabled || cave_opening.is_running()) {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.red.gid_glow);
                } else {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.red.gid_default);
                }
            }

            if let Some(heart_glow) = basic_game_module
                .game_entity_manager
                .heart_wall_glow_map
                .get_mut(&cave_heart_wall.game_state.blue_heart)
            {
                if !cave_opening.is_done() && (blue_heart_enabled || cave_opening.is_running()) {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.default.gid_glow);
                } else {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.default.gid_default);
                }
            }

            if let Some(heart_glow) = basic_game_module
                .game_entity_manager
                .heart_wall_glow_map
                .get_mut(&cave_heart_wall.game_state.yellow_heart)
            {
                if !cave_opening.is_done() && (yellow_heart_enabled || cave_opening.is_running()) {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.yellow.gid_glow);
                } else {
                    heart_glow.set_graphic_id(HeartWallGlow::VARIANTS.yellow.gid_default);
                }
            }
        }

        if let Some(door_to_remove) = door_to_remove_option {
            cave_opening.open(&door_to_remove, basic_game_module);
        }

        if cave_opening.is_running() {
            cave_opening.update(TARGET_FRAME_DURATION, basic_game_module);
        }
    }

    fn update_guest_movement(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        let simulation_config = &basic_game_module.simulation_config;

        let now = Instant::now();
        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get_mut(&slime_1_guest.entity.id)
        {
            let is_guest_standing = Self::is_guest_standing(
                &basic_game_module.simulation,
                &basic_game_module.game_entity_manager.collider_entity_map,
                guest_entity,
            );

            if Self::is_guest_touching_observer(
                &basic_game_module.simulation,
                &basic_game_module.game_entity_manager.collider_entity_map,
                guest_entity,
            ) {
                slime_1_guest.light_touched_time = Some(Instant::now());
            }

            if !is_guest_standing {
                let mut in_air_nudging_force = 0.0;

                if slime_1_guest.guest_input.left {
                    in_air_nudging_force = -simulation_config.guest_in_air_nudging_force;
                } else if slime_1_guest.guest_input.right {
                    in_air_nudging_force = simulation_config.guest_in_air_nudging_force;
                }

                if slime_1_guest.entity.light_id.is_some() {
                    in_air_nudging_force *= simulation_config.light_power_multiplier;
                }

                RapierSimulation::apply_force(
                    &mut basic_game_module.simulation.bodies,
                    guest_entity.physics.body_handle,
                    Vector::new(in_air_nudging_force, 0.0),
                );
            }

            if slime_1_guest.guest_movement.state != GuestMovementState::Idle
                && now
                    .duration_since(slime_1_guest.time_of_last_guest_input)
                    .as_secs()
                    > simulation_config.afk_time
            {
                slime_1_guest.guest_movement.afk();
            }

            match slime_1_guest.guest_movement.state {
                GuestMovementState::HoldJump => {
                    if slime_1_guest.guest_movement.current_animation_progress() >= 1.0
                        && !slime_1_guest.guest_movement.charged_effect_shown
                    {
                        basic_game_module
                            .game_entity_manager
                            .new_show_effects
                            .push(Self::create_charge_effect(slime_1_guest.entity.id.clone()));
                        slime_1_guest.guest_movement.charged_effect_shown = true;
                    }

                    if !slime_1_guest.guest_input.jump {
                        let mut guest_jump_move_force =
                            match slime_1_guest.guest_movement.get_current_direction() {
                                GuestDirections::Left => -simulation_config.guest_jump_move_force,
                                GuestDirections::Right => simulation_config.guest_jump_move_force,
                                GuestDirections::Up => 0.0,
                            };
                        let mut guest_jump_force = simulation_config.guest_jump_force;

                        if slime_1_guest.entity.light_id.is_some() {
                            guest_jump_move_force *= simulation_config.light_power_multiplier;
                            guest_jump_force *= simulation_config.light_power_multiplier;
                        }

                        basic_game_module.simulation.apply_impulse(
                            guest_entity.physics.body_handle,
                            Vector::new(
                                guest_jump_move_force
                                    * slime_1_guest.guest_movement.current_animation_progress(),
                                -guest_jump_force
                                    * slime_1_guest.guest_movement.current_animation_progress(),
                            ),
                        );
                        slime_1_guest.guest_movement.advance_jumping();
                    }

                    if slime_1_guest
                        .guest_movement
                        .ms_since_last_direction_change()
                        > 50
                    {
                        if slime_1_guest.guest_input.left {
                            slime_1_guest.guest_movement.change_direction(
                                match slime_1_guest.guest_movement.get_current_direction() {
                                    GuestDirections::Left => GuestDirections::Left,
                                    GuestDirections::Right => GuestDirections::Up,
                                    GuestDirections::Up => GuestDirections::Left,
                                },
                            );
                        } else if slime_1_guest.guest_input.right {
                            slime_1_guest.guest_movement.change_direction(
                                match slime_1_guest.guest_movement.get_current_direction() {
                                    GuestDirections::Left => GuestDirections::Up,
                                    GuestDirections::Right => GuestDirections::Right,
                                    GuestDirections::Up => GuestDirections::Right,
                                },
                            );
                        } else {
                            slime_1_guest.guest_movement.change_direction(
                                match slime_1_guest.guest_movement.get_current_direction() {
                                    GuestDirections::Left => GuestDirections::Up,
                                    GuestDirections::Right => GuestDirections::Up,
                                    GuestDirections::Up => GuestDirections::Up,
                                },
                            );
                        }
                    }
                }
                GuestMovementState::ReleaseJump => {
                    if slime_1_guest.guest_input.left {
                        slime_1_guest
                            .guest_movement
                            .change_direction(GuestDirections::Left);
                        slime_1_guest.guest_movement.advance_jumping();
                    } else if slime_1_guest.guest_input.right {
                        slime_1_guest
                            .guest_movement
                            .change_direction(GuestDirections::Right);
                        slime_1_guest.guest_movement.advance_jumping();
                    } else if slime_1_guest.guest_input.jump {
                        slime_1_guest.guest_movement.advance_jumping();
                    }
                }
                GuestMovementState::Facing | GuestMovementState::Idle => {
                    if !(slime_1_guest.guest_input.left && slime_1_guest.guest_input.right)
                        && is_guest_standing
                    {
                        if slime_1_guest.guest_input.jump
                            && !slime_1_guest.guest_movement.jumped_in_extended
                        {
                            slime_1_guest
                                .guest_movement
                                .change_direction(GuestDirections::Up);
                            slime_1_guest.guest_movement.advance_jumping();
                        } else if slime_1_guest.guest_input.left {
                            slime_1_guest
                                .guest_movement
                                .change_direction(GuestDirections::Left);
                            slime_1_guest.guest_movement.move_in_direction();
                        } else if slime_1_guest.guest_input.right {
                            slime_1_guest
                                .guest_movement
                                .change_direction(GuestDirections::Right);
                            slime_1_guest.guest_movement.move_in_direction();
                        } else {
                            slime_1_guest.guest_movement.jumped_in_extended = false;
                        }
                    }
                }
                GuestMovementState::Extending => {
                    if slime_1_guest.guest_movement.current_animation_progress() >= 1.0
                        && !slime_1_guest.guest_movement.charged_effect_shown
                    {
                        basic_game_module
                            .game_entity_manager
                            .new_show_effects
                            .push(Self::create_charge_effect(slime_1_guest.entity.id.clone()));
                        slime_1_guest.guest_movement.charged_effect_shown = true;
                    }

                    if slime_1_guest.guest_input.left && slime_1_guest.guest_input.right {
                        slime_1_guest.guest_movement.cancel_move();
                    } else {
                        let mut movement_force: Real = 0.0;

                        if *slime_1_guest.guest_movement.get_current_direction()
                            == GuestDirections::Left
                            && !slime_1_guest.guest_input.left
                        {
                            movement_force = -simulation_config.guest_move_force;
                        }
                        if *slime_1_guest.guest_movement.get_current_direction()
                            == GuestDirections::Right
                            && !slime_1_guest.guest_input.right
                        {
                            movement_force = simulation_config.guest_move_force;
                        }

                        if slime_1_guest.entity.light_id.is_some() {
                            movement_force *= simulation_config.light_power_multiplier;
                        }

                        if movement_force.abs() > 0.0 {
                            basic_game_module.simulation.apply_impulse(
                                guest_entity.physics.body_handle,
                                Vector::new(
                                    movement_force
                                        * slime_1_guest.guest_movement.current_animation_progress(),
                                    0.0,
                                ),
                            );
                            slime_1_guest.guest_movement.move_in_direction();
                        }

                        if !slime_1_guest.guest_input.jump {
                            slime_1_guest.guest_movement.jumped_in_extended = false;
                        }

                        if slime_1_guest.guest_input.jump
                            && !slime_1_guest.guest_movement.jumped_in_extended
                        {
                            slime_1_guest.guest_movement.jumped_in_extended = true;
                            let mut guest_jump_force = simulation_config.guest_jump_force;

                            if slime_1_guest.entity.light_id.is_some() {
                                guest_jump_force *= simulation_config.light_power_multiplier;
                            }

                            basic_game_module.simulation.apply_impulse(
                                guest_entity.physics.body_handle,
                                Vector::new(
                                    0.0,
                                    -(guest_jump_force
                                        * slime_1_guest
                                            .guest_movement
                                            .current_animation_progress()
                                        / 1.3),
                                ),
                            );
                            slime_1_guest.guest_movement.state = GuestMovementState::Moved;
                        }
                    }
                }
                GuestMovementState::Moved => {
                    if slime_1_guest.guest_movement.is_current_animation_done() {
                        slime_1_guest.guest_movement.move_in_direction();
                    }
                }
            }

            if slime_1_guest.entity.light_id.is_some() {
                slime_1_guest
                    .guest_movement
                    .update(TARGET_FRAME_DURATION / 1.5);
            } else {
                slime_1_guest.guest_movement.update(TARGET_FRAME_DURATION);
            }
            guest_entity.set_graphic_id(slime_1_guest.guest_movement.get_current_gid());
        }
    }

    fn is_guest_touching_observer(
        simulation: &RapierSimulation,
        collider_entity_map: &ColliderEntityMap<Slime1GameObject>,
        guest_entity: &GuestEntity,
    ) -> bool {
        for collider_handle in
            simulation.get_intersecting_colliders(guest_entity.physics.collider_handle)
        {
            if collider_entity_map.has(&collider_handle, Slime1GameObject::Observer) {
                return true;
            }
        }

        false
    }

    fn update_heart_touching(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
    ) -> bool {
        let simulation = &basic_game_module.simulation;

        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get(&slime_1_guest.entity.id)
        {
            for collider_handle in
                simulation.get_intersecting_colliders(guest_entity.physics.collider_handle)
            {
                if let Some((id, Slime1GameObject::Heart)) = basic_game_module
                    .game_entity_manager
                    .collider_entity_map
                    .get(&collider_handle)
                {
                    if slime_1_guest.entity.heart_id.is_none() {
                        slime_1_guest.entity.heart_id = Some(id.clone())
                    }
                };
            }
        }

        if let Some(entity_id) = &slime_1_guest.entity.heart_id {
            if let Some(heart_entity) = basic_game_module
                .game_entity_manager
                .remove_heart(entity_id, &mut basic_game_module.simulation)
            {
                if let Some(guest_entity) = basic_game_module
                    .game_entity_manager
                    .guest_map
                    .get_mut(&slime_1_guest.entity.id)
                {
                    guest_entity.game_state.heart_color = heart_entity.game_state.color.clone();
                }

                slime_1_guest.entity.smoll_heart_id = Some(Self::create_smoll_heart(
                    &mut basic_game_module.game_entity_manager,
                    &mut basic_game_module.simulation,
                    &slime_1_guest.entity.id,
                    heart_entity.game_state.color.clone(),
                ));
            }
        }

        false
    }

    fn create_smoll_heart(
        game_entity_manager: &mut Slime1GameEntityManager,
        simulation: &mut RapierSimulation,
        parent_id: &EntityId,
        heart_color: String,
    ) -> EntityId {
        return game_entity_manager.create_heart_smoll(
            HeartSmoll {
                color: "".to_string(),
            },
            Vector::new(0.0, 2.0),
            &HeartSmoll::VARIANTS.default,
            match heart_color.as_str() {
                "red" => HeartSmoll::VARIANTS.default.gid_default.to_string(),
                "blue" => HeartSmoll::VARIANTS.blue.gid_default.to_string(),
                "yellow" => HeartSmoll::VARIANTS.yellow.gid_default.to_string(),
                _ => {
                    warn!("No known heart color?");
                    "0".to_string()
                }
            },
            simulation,
            |entity| {
                entity.render.blending_mode = Some(blending_mode::MULTIPLY);
                entity.render.offset_2d = (0.0, 0.0);
                entity.render.layer = LayerName::BG6;
                entity.parent_entity = Some(parent_id.clone());
            },
        );
    }

    fn update_slime_taking_dead_slimes(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        let guest_dead_map = &mut basic_game_module.game_entity_manager.guest_dead_map;
        let guest_map = &mut basic_game_module.game_entity_manager.guest_map;
        let death_torch_map = &mut basic_game_module.game_entity_manager.death_torch_map;
        let simulation = &mut basic_game_module.simulation;

        if let Some(slime_entity) = guest_map.get_mut(&slime_1_guest.entity.id) {
            basic_game_module
                .game_entity_manager
                .collider_entity_map
                .entities_from_colliders(
                    &simulation.get_intersecting_colliders(slime_entity.physics.collider_handle),
                    death_torch_map,
                    |death_torch| {
                        for guest_dead_entity in guest_dead_map.values_mut() {
                            if guest_dead_entity.game_state.flame == death_torch.id {
                                guest_dead_entity.game_state.slime = slime_entity.id.clone();
                            }
                        }
                    },
                );
        }
    }

    fn update_guest_revival(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        let simulation = &mut basic_game_module.simulation;
        let simulation_config = &basic_game_module.simulation_config;
        let game_entity_manager = &mut basic_game_module.game_entity_manager;
        let death_revive_area_map = &game_entity_manager.death_revive_area_map;
        let mut revive_area_id_option = None;
        if let Some(dead_entity_id) = &slime_1_guest.entity.dead_entity {
            if let Some(guest_entity_dead) =
                game_entity_manager.guest_dead_map.get_mut(dead_entity_id)
            {
                game_entity_manager
                    .collider_entity_map
                    .entities_from_colliders(
                        &simulation
                            .get_intersecting_colliders(guest_entity_dead.physics.collider_handle),
                        &mut game_entity_manager.death_revive_statue_map,
                        |revive_statue| {
                            revive_area_id_option =
                                Some(revive_statue.game_state.revive_area.clone());
                        },
                    );
            }
            if let Some(revive_area_id) = revive_area_id_option {
                let mut spawn_point: Vector<Real> = Vector::new(0.0, 0.0);

                if let Some(revive_area) = game_entity_manager
                    .death_revive_area_map
                    .get(&revive_area_id)
                {
                    spawn_point = GeneralObject::get_random_point(&revive_area.general_object);
                }

                if let Some(guest_dead_entity) =
                    game_entity_manager.remove_guest_dead(dead_entity_id, simulation)
                {
                    let (entity_id, name_plate_id, heart_id_option) = Self::create_guest_entity(
                        &slime_1_guest.guest_name,
                        game_entity_manager,
                        simulation_config,
                        simulation,
                        spawn_point,
                        guest_dead_entity.game_state.heart_color.clone(),
                    );

                    slime_1_guest.entity.heart_id = heart_id_option;

                    game_entity_manager.set_camera_entity_for_guest(
                        slime_1_guest.guest_id.clone(),
                        entity_id.clone(),
                    );

                    if let Err(err) =
                        module_output_sender
                            .game_system_to_guest_sender
                            .send(GuestEvent {
                                guest_id: slime_1_guest.guest_id.clone(),
                                event_type: GameSystemToGuestEvent::SetCamera(
                                    entity_id.clone(),
                                    Slime1Module::module_name(),
                                    CameraSettings::default(),
                                ),
                            })
                    {
                        error!("Could not set camera after guest died, wat!");
                    }

                    slime_1_guest.entity.id = entity_id;
                    slime_1_guest.entity.name_plate_id = name_plate_id;
                }
            }
        }
    }

    fn update_guest_dead(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        let simulation = &mut basic_game_module.simulation;

        if let Some(dead_entity_id) = &slime_1_guest.entity.dead_entity {
            if let Some(guest_entity_dead) = basic_game_module
                .game_entity_manager
                .guest_dead_map
                .get_mut(dead_entity_id)
            {
                let mut movement = Vector::new(0.0, 0.0);
                if slime_1_guest.guest_input.right {
                    movement.x += 1.0;
                }
                if slime_1_guest.guest_input.left {
                    movement.x -= 1.0;
                }
                if slime_1_guest.guest_input.up {
                    movement.y -= 1.0;
                }
                if slime_1_guest.guest_input.down {
                    movement.y += 1.0;
                }

                simulation.move_collider(guest_entity_dead.physics.collider_handle, movement);

                let variant =
                    GuestDead::VARIANTS.get_variant(&slime_1_guest.guest_movement.skin_name);
                if movement.y == 1.0 {
                    guest_entity_dead.set_graphic_id(variant.gid_down);
                } else if movement.y == -1.0 {
                    guest_entity_dead.set_graphic_id(variant.gid_up);
                } else if movement.x == 1.0 {
                    guest_entity_dead.set_graphic_id(variant.gid_right);
                } else if movement.x == -1.0 {
                    guest_entity_dead.set_graphic_id(variant.gid_left);
                } else {
                    guest_entity_dead.set_graphic_id(variant.gid_idle);
                }

                if guest_entity_dead.game_state.slime != "" {
                    if let Some(slime_entity) = basic_game_module
                        .game_entity_manager
                        .guest_map
                        .get_mut(&guest_entity_dead.game_state.slime)
                    {
                        let mut to_slime_direction: Vector<Real> =
                            slime_entity.isometry.translation.vector
                                - guest_entity_dead.isometry.translation.vector;
                        to_slime_direction = to_slime_direction.scale(1.5);
                        simulation.move_collider(
                            guest_entity_dead.physics.collider_handle,
                            to_slime_direction,
                        );
                    } else {
                        guest_entity_dead.game_state.slime = "".to_string();
                    }
                } else if guest_entity_dead.game_state.flame == "" {
                    basic_game_module
                        .game_entity_manager
                        .collider_entity_map
                        .entities_from_colliders(
                            &simulation.get_intersecting_colliders(
                                guest_entity_dead.physics.collider_handle,
                            ),
                            &mut basic_game_module.game_entity_manager.death_area_map,
                            |death_area| {
                                guest_entity_dead.game_state.flame =
                                    death_area.game_state.torch.clone();
                            },
                        );
                } else if guest_entity_dead.game_state.flame != "" {
                    if let Some(flame_entity) = basic_game_module
                        .game_entity_manager
                        .death_torch_map
                        .get_mut(&guest_entity_dead.game_state.flame)
                    {
                        let mut to_flame_direction: Vector<Real> =
                            flame_entity.isometry.translation.vector
                                - guest_entity_dead.isometry.translation.vector;
                        let distance = to_flame_direction.magnitude();
                        if distance > 0.2 {
                            to_flame_direction = to_flame_direction.scale(2.0);
                            simulation.move_collider(
                                guest_entity_dead.physics.collider_handle,
                                to_flame_direction,
                            );
                        }
                    }
                }
            }
        }
    }

    fn update_getting_squished(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        module_output_sender: &mut ModuleOutputSender,
    ) {
        let simulation = &basic_game_module.simulation;
        let mut slime_squished_by_door = false;

        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get(&slime_1_guest.entity.id)
        {
            let contacting_colliders =
                simulation.get_contacting_colliders(guest_entity.physics.collider_handle);

            let guest_aabb = simulation.get_collider_aabb(guest_entity.physics.collider_handle);

            basic_game_module
                .game_entity_manager
                .collider_entity_map
                .entities_from_colliders(
                    &contacting_colliders,
                    &mut basic_game_module.game_entity_manager.door_map,
                    |door| {
                        let door_aabb = simulation.get_collider_aabb(door.physics.collider_handle);
                        let guest_inside_left = ((guest_aabb.maxs.x - door_aabb.mins.x) > 0.008
                            && (guest_aabb.maxs.x < door_aabb.maxs.x));

                        let guest_inside_right = (((door_aabb.maxs.x - guest_aabb.mins.x) > 0.008)
                            && (guest_aabb.mins.x > door_aabb.mins.x));

                        if (door_aabb.maxs.y - guest_aabb.mins.y) > 0.01
                            && (guest_aabb.maxs.y - door_aabb.maxs.y) > 0.01
                            && (guest_inside_left || guest_inside_right)
                        {
                            slime_squished_by_door = true;
                        }
                    },
                );
        }

        if slime_squished_by_door {
            slime_1_guest.squished_timer -= TARGET_FRAME_DURATION;
        } else {
            slime_1_guest.squished_timer = 1000.0;
        }

        if slime_1_guest.squished_timer <= 0.0 {
            if let Err(err) = module_output_sender
                .module_to_system_sender
                .send(ModuleToSystem {
                    guest_id: slime_1_guest.guest_id.clone(),
                    event_type: ModuleToSystemEvent::GlobalMessage(format!(
                        "{} has been Hasorko'd",
                        slime_1_guest.guest_name
                    )),
                })
            {
                error!("Could not send global message, wut");
            }

            Self::kill_guest(
                slime_1_guest,
                &mut basic_game_module.game_entity_manager,
                &mut basic_game_module.simulation,
                module_output_sender,
            );
        }
    }

    fn update_secrets(
        slime_1_guest: &mut Slime1Guest,
        basic_game_module: &mut Slime1BasicGameModule,
        module_output_sender: &mut ModuleOutputSender,
    ) -> bool {
        let simulation = &basic_game_module.simulation;

        if let Some(guest_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get(&slime_1_guest.entity.id)
        {
            basic_game_module
                .game_entity_manager
                .collider_entity_map
                .entities_from_colliders(
                    &simulation.get_intersecting_colliders(guest_entity.physics.collider_handle),
                    &mut basic_game_module.game_entity_manager.secret_map,
                    |secret| {
                        if !slime_1_guest
                            .found_secret_map
                            .contains(&secret.game_state.secret_name)
                        {
                            slime_1_guest
                                .found_secret_map
                                .insert(secret.game_state.secret_name.clone());

                            if let Err(err) =
                                module_output_sender
                                    .module_to_system_sender
                                    .send(GuestEvent {
                                        guest_id: slime_1_guest.guest_id,
                                        event_type: ModuleToSystemEvent::GuestStateChange(
                                            GuestStateChange::FoundSecret(
                                                secret.game_state.secret_name.clone(),
                                                Slime1Module::module_name(),
                                            ),
                                        ),
                                    })
                            {
                                error!("Could not send found secret, that's not good. {:?}", err);
                            }
                        }
                    },
                );
        }

        false
    }

    fn is_guest_standing(
        simulation: &RapierSimulation,
        collider_entity_map: &ColliderEntityMap<Slime1GameObject>,
        guest_entity: &GuestEntity,
    ) -> bool {
        for contact_pair in simulation
            .narrow_phase
            .contacts_with(guest_entity.physics.collider_handle)
        {
            let other_collider_handle =
                if contact_pair.collider1 == guest_entity.physics.collider_handle {
                    contact_pair.collider2
                } else {
                    contact_pair.collider1
                };

            if let Some((
                _id,
                Slime1GameObject::Terrain
                | Slime1GameObject::Guest
                | Slime1GameObject::Door
                | Slime1GameObject::OpeningAreaPlate
                | Slime1GameObject::WallPlatform,
            )) = collider_entity_map.get(&other_collider_handle)
            {
                if let (Some(guest_collider), Some(other_collider)) = (
                    simulation
                        .colliders
                        .get(guest_entity.physics.collider_handle),
                    simulation.colliders.get(other_collider_handle),
                ) {
                    let distance =
                        guest_collider.compute_aabb().maxs.y - other_collider.compute_aabb().mins.y;
                    if distance < 0.01 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn create_new_guest_entity(
        persisted_guest: &PersistedGuest,
        module_enter_slot: &ModuleEnterSlot,
        basic_game_module: &mut Slime1BasicGameModule,
    ) -> (Slime1GuestEntity, GuestMovement) {
        let simulation = &mut basic_game_module.simulation;
        let simulation_config = &basic_game_module.simulation_config;

        let mut spawn_point = Vector::new(0.0, 0.0);
        for spawn_area in basic_game_module
            .game_entity_manager
            .enter_area_map
            .values()
            .filter(|e| e.game_state.slot_id == *module_enter_slot)
        {
            spawn_point = GeneralObject::get_random_point(&spawn_area.general_object);
        }

        if persisted_guest.info.is_observer {
            (
                Slime1GuestEntity {
                    id: basic_game_module.game_entity_manager.create_observer(
                        Observer {},
                        Vector::new(spawn_point.x, spawn_point.y),
                        &Observer::VARIANTS.default,
                        Observer::VARIANTS.default.gid_default.to_string(),
                        &mut basic_game_module.simulation,
                        |entity| {
                            entity.render.blending_mode = Some(blending_mode::ADD);
                        },
                    ),
                    dead_entity: None,
                    heart_id: None,
                    smoll_heart_id: None,
                    light_id: None,
                    name_plate_id: "".to_string(),
                    kind: Slime1GameObject::Observer,
                },
                GuestMovement::new(&persisted_guest.info.slime_skin_name),
            )
        } else {
            let (entity_id, name_plate_id, heart_id_option) = Self::create_guest_entity(
                &persisted_guest.info.display_name,
                &mut basic_game_module.game_entity_manager,
                simulation_config,
                simulation,
                spawn_point,
                "".to_string(),
            );

            (
                Slime1GuestEntity {
                    id: entity_id,
                    name_plate_id,
                    dead_entity: None,
                    light_id: None,
                    heart_id: heart_id_option,
                    smoll_heart_id: None,
                    kind: Slime1GameObject::Guest,
                },
                GuestMovement::new(&persisted_guest.info.slime_skin_name),
            )
        }
    }

    fn create_guest_entity(
        display_name: &String,
        game_entity_manager: &mut Slime1GameEntityManager,
        simulation_config: &Slime1SimulationConfig,
        simulation: &mut RapierSimulation,
        spawn_point: Vector<Real>,
        heart_color: String,
    ) -> (EntityId, EntityId, Option<EntityId>) {
        let entity_id = game_entity_manager.create_guest(
            Guest {
                heart_color: String::new(),
            },
            spawn_point,
            &Guest::VARIANTS.default,
            Guest::VARIANTS.default.gid_face_right.to_string(),
            simulation,
            |entity| {
                entity.game_state.heart_color = heart_color.clone();
            },
        );

        if let Some(entity) = game_entity_manager.guest_map.get_mut(&entity_id) {
            simulation.set_linear_dampening(
                entity.physics.body_handle,
                simulation_config.guest_linear_dampening,
            );
            simulation.set_bounciness(
                entity.physics.collider_handle,
                simulation_config.guest_bounciness,
            );
        }

        let name_plate_id = game_entity_manager.create_guest_nameplate(
            GuestNameplate {},
            Vector::new(0.0, -20.0),
            &PhysicalShape::None,
            "0".to_string(),
            simulation,
            |entity| {
                entity.render.text = display_name.clone();
                entity.render.font_family = "ChevyRay - Express s".to_string();
                entity.render.layer = LayerName::FG5;
                entity.render.center_x = true;
                entity.parent_entity = Some(entity_id.clone());
            },
        );
        (
            entity_id.clone(),
            name_plate_id,
            if !heart_color.is_empty() {
                Some(Self::create_smoll_heart(
                    game_entity_manager,
                    simulation,
                    &entity_id,
                    heart_color,
                ))
            } else {
                None
            },
        )
    }
}

impl GameModuleCommunicationCallbacks<Slime1GameEntityManager, Slime1SimulationConfig>
    for Slime1GameModule
{
    fn update(
        &mut self,
        basic_game_module: &mut Slime1BasicGameModule,
        output_sender: &mut ModuleOutputSender,
    ) {
        for event in self.to_guest_events.drain(..) {
            if let Err(err) = output_sender.game_system_to_guest_sender.send(event) {
                error!("{:?}", err);
            }
        }

        Self::update_heart_wall(&mut self.cave_opening, basic_game_module);
        Self::update_opening_areas(basic_game_module);
        Self::update_doors(basic_game_module);
        Self::update_wall_opening_areas(basic_game_module);
        Self::update_wall_platforms(basic_game_module);
        Self::update_teleporter(
            basic_game_module,
            &mut self.outstanding_guest_teleportations,
        );

        for slime_1_guest in self.slime_1_guest_map.values_mut() {
            Self::check_guest_and_observer_exit(slime_1_guest, basic_game_module, output_sender);

            match slime_1_guest.entity.kind {
                Slime1GameObject::Guest => {
                    Self::update_guest(slime_1_guest, basic_game_module, output_sender);
                }
                Slime1GameObject::Observer => {
                    Self::update_observer(
                        slime_1_guest,
                        basic_game_module,
                        &mut self.guests_in_the_grabs,
                    );
                }
                _ => (),
            }
        }
        for (entity, guest_movement) in self.logged_out_guest_entity_map.values_mut() {
            Self::update_logged_out_guests(entity, guest_movement, basic_game_module);
        }
    }

    fn on_guest_enter(
        &mut self,
        guest_id: &GuestId,
        module_enter_slot: &ModuleEnterSlot,
        _guest_shared_module_state: &HashMap<String, String>,
        persisted_guest: &PersistedGuest,
        basic_game_module: &mut Slime1BasicGameModule,
    ) {
        let (entity, guest_movement) = self
            .logged_out_guest_entity_map
            .remove(&persisted_guest.info.twitch_id)
            .unwrap_or_else(|| {
                Self::create_new_guest_entity(persisted_guest, module_enter_slot, basic_game_module)
            });

        basic_game_module
            .game_entity_manager
            .set_camera_entity_for_guest(*guest_id, entity.id.clone());

        self.slime_1_guest_map.insert(
            *guest_id,
            Slime1Guest {
                entity,
                guest_id: *guest_id,
                guest_name: persisted_guest.info.display_name.clone(),
                time_of_last_guest_input: Instant::now(),
                squished_timer: 1000.0,
                found_secret_map: persisted_guest
                    .secrets_found
                    .iter()
                    .map(|secret| secret.name.clone())
                    .collect(),
                guest_input: GuestInput::new(),
                light_touched_time: None,
                grabbed_by_light: false,
                in_exit_slot: None,
                guest_movement,
            },
        );
    }

    fn on_guest_leave(
        &mut self,
        guest_id: &GuestId,
        persisted_guest: &PersistedGuest,
        _basic_game_module: &mut Slime1BasicGameModule,
    ) {
        if let Some(slime_1_guest) = self.slime_1_guest_map.remove(guest_id) {
            self.logged_out_guest_entity_map.insert(
                persisted_guest.info.twitch_id.clone(),
                (slime_1_guest.entity, slime_1_guest.guest_movement),
            );
        }
    }

    fn on_guest_ready_to_accept_entities(
        &mut self,
        guest_id: &GuestId,
        _basic_game_module: &mut Slime1BasicGameModule,
    ) {
        self.to_guest_events.push(GameSystemToGuest {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::SetMouseInputSchema(
                MouseInputSchema::UpIsJumpAndNoDown,
            ),
        });
    }

    fn on_guest_input(&mut self, guest_id: &GuestId, input: GuestInput) {
        if let Some(slime_1_guest) = self.slime_1_guest_map.get_mut(guest_id) {
            slime_1_guest.guest_input = input;
            slime_1_guest.time_of_last_guest_input = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::entity_manager_generation::imp::generate_entity_manager;

    #[test]
    pub fn generate_slime_1_entity_manager() {
        generate_entity_manager(
            "Slime1",
            "slime_1_module/resources/private/map.tmx",
            "src/slime_1_module/resources/private/objecttypes.xml",
            "src/slime_1_module/game_module/generated/mod.rs",
        );
    }
}
