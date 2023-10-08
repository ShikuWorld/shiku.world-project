use crate::argument_module::def::{ArgumentBasicGameModule, ArgumentSimulationConfig};
use crate::argument_module::game_module::character_movement::{
    CharacterMovement, CharacterMovementState,
};
use crate::argument_module::game_module::generated::{
    AnxietyBar, AnxietyBarVariant, ArgumentGameEntityManager, Guest, IceCrack, IceCrackVariant,
    Stick, StickEntity,
};
use crate::core::entity::def::{Entity, EntityId};
use crate::core::entity_manager::EntityManager;
use crate::core::game_module_communication::GameModuleCommunicationCallbacks;
use crate::core::guest::ModuleEnterSlot;
use std::array::IntoIter;
use std::borrow::BorrowMut;

use crate::core::entity::physics::{Physical, PhysicalShape};
use crate::core::managed_map::ManagedMap;
use crate::core::module::{
    GameSystemToGuest, GameSystemToGuestEvent, GuestEvent, GuestInput, GuestStateChange,
    ModuleOutputSender, ModuleToSystem, ModuleToSystemEvent, MouseInputSchema,
};
use crate::core::module_system::ModuleCommunicationCallbacks;
use crate::core::ring::Cycle;
use crate::core::{Snowflake, TARGET_FRAME_DURATION};
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::GuestId;
use crate::resource_module::map::def::{GeneralObject, LayerName};
use chrono::format::Item;
use log::{debug, error};
use rapier2d::na::Vector2;
use rapier2d::prelude::{Real, Vector};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::slice::Iter;
use std::time::Instant;

pub mod character_movement;
pub mod generated;

pub struct ArgumentGuestEntity {
    pub character_id: String,
    pub anxiety_bar_id: EntityId,
}

#[derive(PartialEq)]
enum StickThrowDirection {
    Left,
    Right,
}

pub struct StickThrow {
    stick_id: EntityId,
    start_pos: Vector2<Real>,
    force_vec: Vector2<Real>,
    direction: StickThrowDirection,
}

pub struct ArgumentGuest {
    pub guest_id: Snowflake,
    pub guest_input: GuestInput,
    pub time_of_last_guest_input: Instant,
    pub movement_animation_time: Real,
    pub entity: ArgumentGuestEntity,
    pub character_movement: CharacterMovement,
    pub data_storage_update: String,
    pub in_exit_slot: Option<String>,
    pub first_joined: bool,
    pub throw_stick: bool,
    pub anxiety: Real,
    pub crack_variants: Cycle<IceCrackVariant, 4>,
}

pub struct ArgumentGameModule {
    pub to_guest_events: Vec<GameSystemToGuest>,
    pub to_system_events: Vec<ModuleToSystem>,
    pub argument_guests: HashMap<GuestId, ArgumentGuest>,
    pub active_stick_throws: ManagedMap<EntityId, StickThrow>,
    pub first_joined: bool,
}

impl ArgumentGameModule {
    pub fn new() -> ArgumentGameModule {
        ArgumentGameModule {
            to_guest_events: Vec::new(),
            to_system_events: Vec::new(),
            argument_guests: HashMap::new(),
            active_stick_throws: ManagedMap::new(),
            first_joined: false,
        }
    }

    pub fn update_anxiety(
        argument_guest: &mut ArgumentGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        let game_entity_manager = &mut basic_game_module.game_entity_manager;
        let simulation = &mut basic_game_module.simulation;
        let guest_map = &game_entity_manager.guest_map;
        let anxiety_bar_map = &mut game_entity_manager.anxiety_bar_map;
        let ice_crack_map = &mut game_entity_manager.ice_crack_map;
        let collider_entity_map = &mut game_entity_manager.collider_entity_map;

        if let Some(guest_entity) = guest_map.get(&argument_guest.entity.character_id) {
            if let Some(anxiety_bar) =
                anxiety_bar_map.get_mut(&argument_guest.entity.anxiety_bar_id)
            {
                let mut touches_ice_crack = false;
                collider_entity_map.entities_from_colliders(
                    &simulation.get_intersecting_colliders(guest_entity.physics.collider_handle),
                    ice_crack_map,
                    |ice_crack| {
                        touches_ice_crack = true;
                    },
                );

                if touches_ice_crack {
                    argument_guest.anxiety += 0.1;
                    if argument_guest.anxiety > 100.0 {
                        argument_guest.anxiety = 100.0;
                    }
                }
                anxiety_bar.set_width(argument_guest.anxiety as u32);
            }
        }
    }

    fn _wants_forwards(mov_vec: Vector2<Real>, right_char: bool) -> bool {
        (mov_vec.x > 0.0 && !right_char)
            || (mov_vec.x < 0.0 && right_char)
            || (mov_vec.x == 0.0 && mov_vec.y != 0.0)
    }

    fn _wants_backwards(mov_vec: Vector2<Real>, right_char: bool) -> bool {
        (mov_vec.x < 0.0 && !right_char) || (mov_vec.x > 0.0 && right_char)
    }

    pub fn create_stick_to_throw(
        argument_guest: &mut ArgumentGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) -> StickThrow {
        let mut stick_start_position = Vector2::new(0.0, 0.0);
        let mut force_vec = Vector2::new(0.0, basic_game_module.simulation_config.throw_force_y);
        let mut direction = StickThrowDirection::Right;
        if let Some(guest_character) = basic_game_module
            .game_entity_manager
            .guest_map
            .get_mut(&argument_guest.entity.character_id)
        {
            stick_start_position.x = guest_character.isometry.translation.x;
            stick_start_position.y = guest_character.isometry.translation.y
                - (basic_game_module.simulation_config.stick_offset_y
                    / basic_game_module.simulation.simulation_scaling_factor);

            if guest_character.game_state.flipped {
                direction = StickThrowDirection::Left;
                stick_start_position.x -= basic_game_module.simulation_config.stick_offset_x
                    / basic_game_module.simulation.simulation_scaling_factor;
                force_vec.x = -basic_game_module.simulation_config.throw_force_x;
            } else {
                direction = StickThrowDirection::Right;
                stick_start_position.x += basic_game_module.simulation_config.stick_offset_x
                    / basic_game_module.simulation.simulation_scaling_factor;
                force_vec.x = basic_game_module.simulation_config.throw_force_x;
            };
        }

        let stick_id = basic_game_module.game_entity_manager.create_stick(
            Stick {
                thrown_by: "".into(),
            },
            Vector2::new(
                stick_start_position.x * basic_game_module.simulation.simulation_scaling_factor,
                stick_start_position.y * basic_game_module.simulation.simulation_scaling_factor,
            ),
            &Stick::VARIANTS.default,
            if direction == StickThrowDirection::Right {
                Stick::VARIANTS.default.gid_default.into()
            } else {
                Stick::VARIANTS.default.gid_backwards.into()
            },
            &mut basic_game_module.simulation,
            |stick| {
                stick.render.layer = LayerName::FG10;
            },
        );

        StickThrow {
            direction,
            stick_id,
            force_vec,
            start_pos: stick_start_position,
        }
    }

    pub fn update_stick_throwing(
        stick_throws: &mut ManagedMap<EntityId, StickThrow>,
        argument_guest: &mut ArgumentGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        if argument_guest.throw_stick {
            argument_guest.throw_stick = false;
            let stick_throw = Self::create_stick_to_throw(argument_guest, basic_game_module);
            stick_throws.insert(stick_throw.stick_id.clone(), stick_throw);
        }

        for stick_throw_entry in stick_throws.entries_mut() {
            let mut stick_throw = &mut stick_throw_entry.data;
            if let Some(stick) = basic_game_module
                .game_entity_manager
                .stick_map
                .get(&stick_throw.stick_id)
            {
                basic_game_module
                    .simulation
                    .move_collider(stick.physics.collider_handle, stick_throw.force_vec);
                stick_throw.force_vec.y += basic_game_module.simulation_config.stick_gravity;

                if (stick.isometry.translation.y - stick_throw.start_pos.y) > 0.01 {
                    stick_throw_entry.mark_for_deletion();
                }
            }
        }

        stick_throws.delete(|stick_throw| {
            if let Some(stick) = basic_game_module
                .game_entity_manager
                .remove_stick(&stick_throw.stick_id, &mut basic_game_module.simulation)
            {
                let crack_pos = stick.isometry.translation;
                let scf = basic_game_module.simulation.simulation_scaling_factor;
                if let Some(variant) = argument_guest.crack_variants.next() {
                    basic_game_module.game_entity_manager.create_ice_crack(
                        IceCrack {},
                        Vector::new(crack_pos.x * scf, crack_pos.y * scf),
                        &variant,
                        variant.gid_crack.into(),
                        &mut basic_game_module.simulation,
                        |crack| {},
                    );
                }
            }
        });
    }

    pub fn update_guest_movement(
        argument_guest: &mut ArgumentGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        if let Some(guest_character) = basic_game_module
            .game_entity_manager
            .guest_map
            .get_mut(&argument_guest.entity.character_id)
        {
            let mut mov_vec = Self::_mov_vec_from_input(&argument_guest.guest_input);
            let right_char = guest_character.game_state.flipped;
            match argument_guest.character_movement.state {
                CharacterMovementState::Hitting => {
                    if argument_guest
                        .character_movement
                        .is_current_animation_done()
                    {
                        argument_guest.character_movement.hit_done();
                    }
                }
                CharacterMovementState::Idle | CharacterMovementState::Idle2 => {
                    if argument_guest.guest_input.action_1 {
                        argument_guest.character_movement.start_throw();
                    } else if argument_guest.guest_input.action_2 {
                        argument_guest.character_movement.hit();
                    } else if Self::_wants_forwards(mov_vec, right_char) {
                        argument_guest.character_movement.move_forwards();
                    } else if Self::_wants_backwards(mov_vec, right_char) {
                        argument_guest.character_movement.move_backwards();
                    }
                }
                CharacterMovementState::MoveForwards => {
                    if argument_guest.guest_input.action_1 {
                        argument_guest.character_movement.start_throw();
                    } else if Self::_wants_backwards(mov_vec, right_char) {
                        argument_guest.character_movement.move_backwards();
                    } else if mov_vec.x.abs() + mov_vec.y.abs() == 0.0 {
                        argument_guest.character_movement.stop_move();
                    }
                }
                CharacterMovementState::MoveBackwards => {
                    mov_vec.x /= 2.0;
                    mov_vec.y /= 2.0;
                    if argument_guest.guest_input.action_1 {
                        argument_guest.character_movement.start_throw();
                    } else if Self::_wants_forwards(mov_vec, right_char) {
                        argument_guest.character_movement.move_forwards();
                    } else if mov_vec.x.abs() + mov_vec.y.abs() == 0.0 {
                        argument_guest.character_movement.stop_move();
                    }
                }
                CharacterMovementState::StartThrowing => {
                    mov_vec.x = 0.0;
                    mov_vec.y = 0.0;
                    if !argument_guest.guest_input.action_1 {
                        argument_guest.character_movement.cancel_throw();
                    } else if argument_guest
                        .character_movement
                        .is_current_animation_done()
                    {
                        argument_guest.character_movement.hold_throw();
                    }
                }
                CharacterMovementState::HoldThrow => {
                    mov_vec.x = 0.0;
                    mov_vec.y = 0.0;
                    if !argument_guest.guest_input.action_1 {
                        argument_guest.character_movement.throw();
                        argument_guest.throw_stick = true;
                    }
                }
                CharacterMovementState::Throw => {
                    mov_vec.x = 0.0;
                    mov_vec.y = 0.0;
                    if argument_guest
                        .character_movement
                        .is_current_animation_done()
                    {
                        argument_guest.character_movement.throw_done();
                    }
                }
                CharacterMovementState::Hitting => {}
            }

            basic_game_module
                .simulation
                .s_set_velocity(guest_character.physics.body_handle, mov_vec);

            argument_guest
                .character_movement
                .update(TARGET_FRAME_DURATION);
            guest_character.set_graphic_id(argument_guest.character_movement.get_current_gid());
        }
    }

    fn _mov_vec_from_input(guest_input: &GuestInput) -> Vector2<Real> {
        let mut vec = Vector2::new(0.0, 0.0);
        if guest_input.left {
            vec.x -= 3.0;
        }

        if guest_input.right {
            vec.x += 3.0;
        }

        if guest_input.up {
            vec.y -= 3.0;
        }

        if guest_input.down {
            vec.y += 3.0;
        }

        vec
    }

    pub fn create_argument_guest(
        module_enter_slot: &ModuleEnterSlot,
        basic_game_module: &mut ArgumentBasicGameModule,
        first_joined: bool,
    ) -> (EntityId, EntityId) {
        let mut spawn_point = Vector::new(0.0, 0.0);
        for spawn_area in basic_game_module
            .game_entity_manager
            .enter_area_map
            .values()
            .filter(|e| e.game_state.slot_id == *module_enter_slot)
        {
            if spawn_area.game_state.first == first_joined {
                spawn_point = GeneralObject::get_random_point(&spawn_area.general_object);
            }
        }

        let slime_entity_id = basic_game_module.game_entity_manager.create_guest(
            Guest {
                flipped: first_joined,
            },
            spawn_point,
            &Guest::VARIANTS.default,
            Guest::VARIANTS.default.gid_default.to_string(),
            &mut basic_game_module.simulation,
            |entity| {
                entity.render.scale = (if first_joined { -1.0 } else { 1.0 }, 1.0);
            },
        );

        let anxiety_bar_id = basic_game_module.game_entity_manager.create_anxiety_bar(
            AnxietyBar {},
            Vector::new(if first_joined { -70.0 } else { 70.0 }, 100.0),
            &AnxietyBar::VARIANTS.default,
            AnxietyBar::VARIANTS.default.gid_red.into(),
            &mut basic_game_module.simulation,
            |anxiety_bar| {
                anxiety_bar.render.layer = LayerName::Menu;
                anxiety_bar.set_width(1);
                anxiety_bar.set_tiled(true);
            },
        );

        (slime_entity_id, anxiety_bar_id)
    }

    fn check_guest_exit(
        argument_guest: &mut ArgumentGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
        output_sender: &mut ModuleOutputSender,
    ) {
        if argument_guest.in_exit_slot.is_some() {
            return;
        }

        if let Some(character_entity) = basic_game_module
            .game_entity_manager
            .guest_map
            .get_mut(&argument_guest.entity.character_id)
        {
            basic_game_module
                .game_entity_manager
                .collider_entity_map
                .entities_from_colliders(
                    &basic_game_module
                        .simulation
                        .get_intersecting_colliders(character_entity.physics.collider_handle),
                    &mut basic_game_module.game_entity_manager.exit_area_map,
                    |exit_area| {
                        argument_guest.in_exit_slot = Some(exit_area.game_state.slot_id.clone());
                    },
                );

            if let Some(module_exit_slot) = &argument_guest.in_exit_slot {
                if let Err(err) = output_sender.module_to_system_sender.send(GuestEvent {
                    guest_id: argument_guest.guest_id.clone(),
                    event_type: ModuleToSystemEvent::GuestStateChange(
                        GuestStateChange::ExitModule(module_exit_slot.clone()),
                    ),
                }) {
                    error!(
                        "Could not send exit module event, this is very bad! {:?}",
                        err
                    );
                }
            }
        }
    }
}

impl GameModuleCommunicationCallbacks<ArgumentGameEntityManager, ArgumentSimulationConfig>
    for ArgumentGameModule
{
    fn update(
        &mut self,
        basic_game_module: &mut ArgumentBasicGameModule,
        output_sender: &mut ModuleOutputSender,
    ) {
        for event in self.to_guest_events.drain(..) {
            if let Err(err) = output_sender.game_system_to_guest_sender.send(event) {
                error!("{:?}", err);
            }
        }

        for event in self.to_system_events.drain(..) {
            if let Err(err) = output_sender.module_to_system_sender.send(event) {
                error!("{:?}", err);
            }
        }

        for argument_guest in self.argument_guests.values_mut() {
            Self::update_guest_movement(argument_guest, basic_game_module);
            Self::update_stick_throwing(
                &mut self.active_stick_throws,
                argument_guest,
                basic_game_module,
            );
            Self::update_anxiety(argument_guest, basic_game_module);
            Self::check_guest_exit(argument_guest, basic_game_module, output_sender);
        }
    }

    fn on_guest_enter(
        &mut self,
        guest_id: &GuestId,
        module_enter_slot: &ModuleEnterSlot,
        _guest_shared_module_state: &HashMap<String, String>,
        _persisted_guest: &PersistedGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        let (character_id, anxiety_bar_id) =
            Self::create_argument_guest(module_enter_slot, basic_game_module, self.first_joined);

        basic_game_module
            .game_entity_manager
            .set_camera_entity_for_guest(guest_id.clone(), character_id.clone());

        self.argument_guests.insert(
            guest_id.clone(),
            ArgumentGuest {
                first_joined: self.first_joined,
                guest_id: guest_id.clone(),
                entity: ArgumentGuestEntity {
                    character_id,
                    anxiety_bar_id,
                },
                anxiety: 0.0,
                guest_input: GuestInput::new(),
                data_storage_update: String::new(),
                movement_animation_time: 0.0,
                character_movement: CharacterMovement::new(
                    &(if self.first_joined {
                        "default".into()
                    } else {
                        "red".into()
                    }),
                ),
                crack_variants: Cycle::new([
                    IceCrack::VARIANTS.c1,
                    IceCrack::VARIANTS.c2,
                    IceCrack::VARIANTS.c3,
                    IceCrack::VARIANTS.c4,
                ]),
                in_exit_slot: None,
                throw_stick: false,
                time_of_last_guest_input: Instant::now(),
            },
        );

        self.first_joined = true;
    }

    fn on_guest_leave(
        &mut self,
        guest_id: &GuestId,
        _persisted_guest: &PersistedGuest,
        basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        if let Some(guest) = self.argument_guests.remove(guest_id) {
            basic_game_module.game_entity_manager.remove_guest(
                &guest.entity.character_id,
                &mut basic_game_module.simulation,
            );
        }
    }

    fn on_guest_ready_to_accept_entities(
        &mut self,
        guest_id: &GuestId,
        _basic_game_module: &mut ArgumentBasicGameModule,
    ) {
        self.to_guest_events.push(GameSystemToGuest {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::SetMouseInputSchema(
                MouseInputSchema::PurelyDirectionalNoJump,
            ),
        });
    }

    fn on_guest_input(&mut self, guest_id: &GuestId, input: GuestInput) {
        if let Some(argument_guest) = self.argument_guests.get_mut(guest_id) {
            argument_guest.guest_input = input;
            argument_guest.time_of_last_guest_input = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::entity_manager_generation::imp::generate_entity_manager;

    #[test]
    pub fn generate_argument_entity_manager() {
        generate_entity_manager(
            "Argument",
            "argument_module/resources/private/argument.tmx",
            "src/argument_module/resources/private/objecttypes.xml",
            "src/argument_module/game_module/generated/mod.rs",
        );
    }
}
