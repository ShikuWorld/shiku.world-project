use crate::core::entity::def::EntityId;
use crate::core::entity_manager::{ColliderEntityMap, EntityManager};
use crate::core::game_module_communication::GameModuleCommunicationCallbacks;
use crate::core::guest::ModuleEnterSlot;
use crate::core::medium_data_storage::{
    MediumDataStorage, MediumDataStorageGuestInfo, SecretFoundEntry,
};
use crate::core::module::{
    GameSystemToGuest, GameSystemToGuestEvent, GuestInput, ModuleName, ModuleOutputSender,
    ModuleToSystem, MouseInputSchema,
};
use crate::core::module_system::ModuleCommunicationCallbacks;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::TARGET_FRAME_DURATION;
use crate::lobby_module::def::{LobbyBasicGameModule, LobbyModule, LobbySimulationConfig};
use crate::lobby_module::game_module::generated::{
    ExitAreaEntity, Guest, GuestEntity, LobbyGameEntityManager, LobbyGameObject,
};
use crate::lobby_module::game_module::module_enter_process::ModuleEnterProcesses;
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::GuestId;
use crate::resource_module::map::def::GeneralObject;
use log::{debug, error};
use rapier2d::prelude::{Real, Vector};
use std::collections::HashMap;
use std::time::Instant;

pub mod generated;
pub mod module_enter_process;

pub struct LobbyGuestEntity {
    pub slime_entity_id: String,
}

pub struct LobbyGuest {
    pub guest_input: GuestInput,
    pub time_of_last_guest_input: Instant,
    pub movement_animation_time: Real,
    pub entity: LobbyGuestEntity,
    pub data_storage_update: String,
    pub about_to_leave: bool,
}

pub struct LobbyGameModule {
    pub to_guest_events: Vec<GameSystemToGuest>,
    pub to_system_events: Vec<ModuleToSystem>,
    pub lobby_guests: HashMap<GuestId, LobbyGuest>,
    pub enter_processes: ModuleEnterProcesses,
    pub some_timer: Instant,
}

impl LobbyGameModule {
    pub fn new(basic_game_module: &LobbyBasicGameModule) -> LobbyGameModule {
        let mut enter_processes = ModuleEnterProcesses::new();

        enter_processes.add_enter_process(LobbyModule::EXIT_SLOTS.to_module_exit_1.into(), 1, true);
        enter_processes.add_enter_process(
            LobbyModule::EXIT_SLOTS.to_module_exit_2.into(),
            1,
            false,
        );

        for exit_area in basic_game_module.game_entity_manager.exit_area_map.values() {
            enter_processes.add_exit_area(&exit_area.game_state.slot_id, exit_area.id.clone());
        }

        LobbyGameModule {
            to_guest_events: Vec::new(),
            to_system_events: Vec::new(),
            lobby_guests: HashMap::new(),
            enter_processes,
            some_timer: Instant::now(),
        }
    }

    fn create_lobby_guest_entity(
        module_enter_slot: &ModuleEnterSlot,
        basic_game_module: &mut LobbyBasicGameModule,
    ) -> EntityId {
        let mut spawn_point = Vector::new(0.0, 0.0);
        for spawn_area in basic_game_module
            .game_entity_manager
            .enter_area_map
            .values()
            .filter(|e| e.game_state.slot_id == *module_enter_slot)
        {
            spawn_point = GeneralObject::get_random_point(&spawn_area.general_object);
        }

        let slime_entity_id = basic_game_module.game_entity_manager.create_guest(
            Guest {},
            spawn_point,
            &Guest::VARIANTS.default,
            Guest::VARIANTS.default.gid_default.to_string(),
            &mut basic_game_module.simulation,
            |_entity| {},
        );
        slime_entity_id
    }

    fn create_lobby_menu_data_storage(persisted_guest: &PersistedGuest) -> String {
        let data_storage_update: String = serde_json::to_string(&MediumDataStorage {
            current_guest_info: Some(MediumDataStorageGuestInfo {
                times_joined: persisted_guest.info.times_joined,
                guests_online: 0,
                guest_name: persisted_guest.info.display_name.clone(),
                secrets_found_count: persisted_guest.secrets_found.len() as i32,
                secrets_found_map: persisted_guest
                    .secrets_found
                    .iter()
                    .map(|s| {
                        (
                            s.name.clone(),
                            SecretFoundEntry {
                                name: s.name.clone(),
                                date: s.date.timestamp(),
                            },
                        )
                    })
                    .collect(),
            }),
        })
        .unwrap_or_else(|_err| "{}".to_string());
        data_storage_update
    }

    fn update_guest_movement(
        simulation: &mut RapierSimulation,
        lobby_guest: &mut LobbyGuest,
        slime_entity: &mut GuestEntity,
        vel_x_add: Real,
        vel_y_add: Real,
    ) {
        let mut vel_x = 0.0;
        let mut vel_y = 0.0;

        if lobby_guest.guest_input.left {
            vel_x -= 1.0;
        }

        if lobby_guest.guest_input.right {
            vel_x += 1.0;
        }

        if lobby_guest.guest_input.up {
            vel_y -= 1.0;
        }

        if lobby_guest.guest_input.jump {
            vel_y -= 1.0;
        }

        if lobby_guest.guest_input.down {
            vel_y += 1.0;
        }

        if vel_y == 0.0 && vel_x == 0.0 {
            slime_entity.set_graphic_id(Guest::VARIANTS.default.gid_default);
            simulation.s_set_velocity(
                slime_entity.physics.body_handle,
                Vector::new(vel_x_add, vel_y_add),
            );
        } else {
            if lobby_guest.movement_animation_time <= 100.0 {
                simulation.s_set_velocity(
                    slime_entity.physics.body_handle,
                    Vector::new(vel_x_add + (vel_x / 3.0), vel_y_add + (vel_y / 3.0)),
                );
                slime_entity.set_graphic_id(Guest::VARIANTS.default.gid_move);
            } else {
                simulation.s_set_velocity(
                    slime_entity.physics.body_handle,
                    Vector::new(vel_x + vel_x_add, vel_y + vel_y_add),
                );
                slime_entity.set_graphic_id(Guest::VARIANTS.default.gid_default)
            }

            if lobby_guest.movement_animation_time >= 200.0 {
                lobby_guest.movement_animation_time = 0.0;
            }

            lobby_guest.movement_animation_time += TARGET_FRAME_DURATION;
        }
    }

    fn update_guest_in_exit_area(
        collider_entity_map: &mut ColliderEntityMap<LobbyGameObject>,
        simulation: &mut RapierSimulation,
        exit_area_map: &mut HashMap<EntityId, ExitAreaEntity>,
        enter_processes: &mut ModuleEnterProcesses,
        guest_id: &GuestId,
        slime_entity: &mut GuestEntity,
    ) {
        let mut is_guest_in_exit_area = false;
        collider_entity_map.entities_from_colliders(
            &simulation.get_intersecting_colliders(slime_entity.physics.collider_handle),
            exit_area_map,
            |exit_area| {
                if enter_processes.is_guest_being_processed(guest_id) {
                    return;
                }

                enter_processes.add_guest_to_exit_area(
                    &exit_area.game_state.slot_id,
                    &exit_area.id,
                    guest_id,
                );
                is_guest_in_exit_area = true;
            },
        );

        if !is_guest_in_exit_area {
            enter_processes.remove_guest_from_its_area(&guest_id);
        }
    }

    fn send_guest_count_to_guests(&mut self) {
        let lobby_guest_count = self.lobby_guests.len();
        for guest_id in self.lobby_guests.keys() {
            self.to_guest_events.push(GameSystemToGuest {
                guest_id: *guest_id,
                event_type: GameSystemToGuestEvent::UpdateDataStore(format!(
                    "{{\"current_guest_info\": {{ \"guests_online\": {}}} }}",
                    lobby_guest_count
                )),
            });
        }
    }
}

impl GameModuleCommunicationCallbacks<LobbyGameEntityManager, LobbySimulationConfig>
    for LobbyGameModule
{
    fn update(
        &mut self,
        basic_game_module: &mut LobbyBasicGameModule,
        output_sender: &mut ModuleOutputSender,
    ) {
        let collider_entity_map = &mut basic_game_module.game_entity_manager.collider_entity_map;
        let simulation = &mut basic_game_module.simulation;
        let exit_area_map = &mut basic_game_module.game_entity_manager.exit_area_map;
        let rolling_map = &mut basic_game_module.game_entity_manager.rolling_map;
        let enter_processes = &mut self.enter_processes;

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

        for (guest_id, lobby_guest) in self.lobby_guests.iter_mut() {
            let mut vel_x: Real = 0.0;
            let mut vel_y: Real = 0.0;
            if let Some(slime_entity) = basic_game_module
                .game_entity_manager
                .guest_map
                .get_mut(&lobby_guest.entity.slime_entity_id)
            {
                collider_entity_map.entities_from_colliders(
                    &simulation.get_intersecting_colliders(slime_entity.physics.collider_handle),
                    rolling_map,
                    |rolling| {
                        match rolling.game_state.direction.as_str() {
                            "top" => {
                                vel_y = -2.0;
                            }
                            "bottom" => {
                                vel_y = 2.0;
                            }
                            "left" => {
                                vel_x = -2.0;
                            }
                            "right" => {
                                vel_x = 2.0;
                            }
                            _ => {}
                        };
                    },
                );

                Self::update_guest_movement(simulation, lobby_guest, slime_entity, vel_x, vel_y);

                Self::update_guest_in_exit_area(
                    collider_entity_map,
                    simulation,
                    exit_area_map,
                    enter_processes,
                    guest_id,
                    slime_entity,
                );
            }
        }

        enter_processes.update_enter_processes(output_sender);

        for process in enter_processes.processes.values() {
            for exit_area_id in process.exit_area_guest_map.keys() {
                if let Some(exit_area) = exit_area_map.get(exit_area_id) {
                    if let Some(text) = basic_game_module
                        .game_entity_manager
                        .text_map
                        .get_mut(&exit_area.game_state.info)
                    {
                        text.set_text(&process.info_text);
                    }
                }
            }
        }
    }

    fn on_guest_enter(
        &mut self,
        guest_id: &GuestId,
        module_enter_slot: &ModuleEnterSlot,
        _guest_shared_module_state: &HashMap<String, String>,
        persisted_guest: &PersistedGuest,
        basic_game_module: &mut LobbyBasicGameModule,
    ) {
        let data_storage_update = Self::create_lobby_menu_data_storage(persisted_guest);

        let slime_entity_id = Self::create_lobby_guest_entity(module_enter_slot, basic_game_module);

        basic_game_module
            .game_entity_manager
            .set_camera_entity_for_guest(guest_id.clone(), slime_entity_id.clone());

        self.lobby_guests.insert(
            *guest_id,
            LobbyGuest {
                data_storage_update,
                movement_animation_time: 0.0,
                guest_input: GuestInput::new(),
                time_of_last_guest_input: Instant::now(),
                entity: LobbyGuestEntity { slime_entity_id },
                about_to_leave: false,
            },
        );

        self.send_guest_count_to_guests();
    }

    fn on_guest_leave(
        &mut self,
        guest_id: &GuestId,
        _persisted_guest: &PersistedGuest,
        basic_game_module: &mut LobbyBasicGameModule,
    ) {
        if let Some(lobby_guest) = self.lobby_guests.remove(guest_id) {
            basic_game_module.game_entity_manager.remove_guest(
                &lobby_guest.entity.slime_entity_id,
                &mut basic_game_module.simulation,
            );
            self.enter_processes.on_guest_leave(guest_id);
        }

        self.send_guest_count_to_guests();
    }

    fn on_guest_ready_to_accept_entities(
        &mut self,
        guest_id: &GuestId,
        _basic_game_module: &mut LobbyBasicGameModule,
    ) {
        let lobby_guest_count = self.lobby_guests.len();
        if let Some(lobby_guest) = self.lobby_guests.get(guest_id) {
            self.to_guest_events.push(GameSystemToGuest {
                guest_id: *guest_id,
                event_type: GameSystemToGuestEvent::UpdateDataStore(
                    lobby_guest.data_storage_update.clone(),
                ),
            });

            self.to_guest_events.push(GameSystemToGuest {
                guest_id: *guest_id,
                event_type: GameSystemToGuestEvent::UpdateDataStore(format!(
                    "{{\"current_guest_info\": {{ \"guests_online\": {}}} }}",
                    lobby_guest_count
                )),
            });

            self.to_guest_events.push(GameSystemToGuest {
                guest_id: *guest_id,
                event_type: GameSystemToGuestEvent::SetMouseInputSchema(
                    MouseInputSchema::PurelyDirectionalNoJump,
                ),
            });
        }
    }

    fn on_guest_input(&mut self, guest_id: &GuestId, input: GuestInput) {
        if let Some(lobby_guest) = self.lobby_guests.get_mut(guest_id) {
            lobby_guest.guest_input = input;
            lobby_guest.time_of_last_guest_input = Instant::now();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::entity_manager_generation::imp::generate_entity_manager;

    #[test]
    pub fn generate_lobby_entity_manager() {
        generate_entity_manager(
            "Lobby",
            "lobby_module/resources/private/lobby.tmx",
            "src/lobby_module/resources/private/objecttypes.xml",
            "src/lobby_module/game_module/generated/mod.rs",
        );
    }
}
