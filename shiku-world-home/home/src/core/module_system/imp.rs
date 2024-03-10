use std::collections::HashMap;
use std::path::PathBuf;

use apecs::World as ApecsWorld;
use flume::Sender;
use log::{debug, error};
use tokio::time::Instant;

use crate::core::blueprint::def::{BlueprintService, Chunk, GameMap, LayerKind, Module, ModuleId, Scene, TerrainParams};
use crate::core::guest::ActorId;
use crate::core::guest::{Admin, Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication_input, EnterFailedState, EnterSuccessState, GameSystemToGuest,
    GameSystemToGuestEvent, GuestEvent, GuestToModule, LeaveFailedState, LeaveSuccessState,
    ModuleInputSender, ModuleInstanceEvent, ModuleOutputSender,
};
use crate::core::module::{GuestInput, GuestToModuleEvent};
use crate::core::module_system::def::{
    DynamicGameModule, GuestCommunication, GuestMap, ModuleAdmin, ModuleCommunication, ModuleGuest,
};
use crate::core::module_system::error::{CreateWorldError, DestroyWorldError};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::{cantor_pair, send_and_log_error, send_and_log_error_custom, LazyHashmapSet};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::module_system::world::{World, WorldId};

impl DynamicGameModule {
    pub fn create(
        instance_id: GameInstanceId,
        module: &Module,
        module_output_sender: ModuleOutputSender,
    ) -> (DynamicGameModule, ModuleInputSender) {
        let (module_input_sender, module_input_receiver) = create_module_communication_input();
        let mut dynamic_module = DynamicGameModule {
            world_map: HashMap::new(),
            guests: HashMap::new(),
            admins: HashMap::new(),
            guest_to_world: HashMap::new(),
            admin_to_world: LazyHashmapSet::new(),
            world_to_admin: LazyHashmapSet::new(),
            world_to_guest: LazyHashmapSet::new(),
            module_communication: ModuleCommunication::new(
                module_input_receiver,
                module_output_sender,
            ),
            module_id: module.id.clone(),
            instance_id,
        };
        let game_maps = BlueprintService::load_all_maps_for_module(module).unwrap_or_else(|err| {
            error!("Could not load maps for module to create worlds {:?}", err);
            Vec::new()
        });
        for game_map in game_maps {
            if let Err(err) = dynamic_module.create_world(&game_map) {
                error!("Could not create world '{}': {:?}", game_map.name, err);
            }
        }

        (dynamic_module, module_input_sender)
    }

    pub fn create_world(&mut self, game_map: &GameMap) -> Result<WorldId, CreateWorldError> {
        if self.world_map.contains_key(&game_map.world_id) {
            return Err(CreateWorldError::DidAlreadyExist);
        }

        self.world_map.insert(
            game_map.world_id.clone(),
            World::new(&game_map)?,
        );
        self.world_to_admin.init(game_map.world_id.clone());
        self.world_to_guest.init(game_map.world_id.clone());
        Ok(game_map.world_id.clone())
    }

    pub fn get_terrain_params(&self, world_id: &WorldId) -> Option<TerrainParams> {
        if let Some(world) = self.world_map.get(world_id) {
            return Some(world.terrain_params.clone());
        }
        None
    }

    pub fn destroy_world(&mut self, game_map: &GameMap) -> Result<WorldId, DestroyWorldError> {
        if self.world_to_guest.len(&game_map.world_id) > 0
            || self.world_to_admin.len(&game_map.world_id) > 0
        {
            return Err(DestroyWorldError::StillHasInhabitants);
        }
        if !self.world_map.contains_key(&game_map.world_id) {
            return Err(DestroyWorldError::DidNotExist);
        }
        self.world_map.remove(&game_map.world_id);
        self.world_to_admin.remove(&game_map.world_id);
        self.world_to_guest.remove(&game_map.world_id);
        Ok(game_map.world_id.clone())
    }

    fn set_guest_input(guests: &mut GuestMap, guest_id: &ActorId, input: GuestInput) {
        if let Some(guest) = guests.get_mut(guest_id) {
            guest.guest_input = input;
            guest.last_input_time = Instant::now();
        }
    }

    fn set_resources_loaded(guests: &mut GuestMap, guest_id: &ActorId) {
        if let Some(guest) = guests.get_mut(guest_id) {
            guest.guest_com.resources_loaded = true;
        }
    }

    pub fn update(&mut self, module: &Module) {
        self.handle_guest_events(module);
    }

    pub fn actor_disconnected(&mut self, actor_id: &ActorId) {
        if let Some(guest) = self.guests.get_mut(actor_id) {
            guest.guest_com.connected = false;
        }
        if let Some(admin) = self.admins.get_mut(actor_id) {
            admin.connected = false;
        }
    }

    pub fn update_world_map(&mut self, world_id: &WorldId, layer_kind: &LayerKind, chunk: &Chunk) {
        //TODO: Update terrain if layer is terrain

        self.world_map
            .get_mut(world_id)
            .and_then(|world| world.terrain.get_mut(layer_kind))
            .and_then(|chunks| {
                chunks.insert(
                    cantor_pair(chunk.position.0, chunk.position.1),
                    chunk.clone(),
                )
            });
        let mut terrain_update = ModuleInstanceEvent {
            world_id: None,
            module_id: self.module_id.clone(),
            instance_id: self.instance_id.clone(),
            event_type: GameSystemToGuestEvent::ShowTerrain(vec![(
                layer_kind.clone(),
                vec![chunk.clone()],
            )]),
        };
        if let (Some(guest_ids), Some(admin_ids)) = (
            self.world_to_guest.hashset(world_id),
            self.world_to_admin.hashset(world_id),
        ) {
            let send_terrain_update =
                &mut |actor_id: ActorId, update: ModuleInstanceEvent<GameSystemToGuestEvent>| {
                    send_and_log_error_custom(
                        &mut self
                            .module_communication
                            .output_sender
                            .game_system_to_guest_sender,
                        GuestEvent {
                            guest_id: actor_id,
                            event_type: update,
                        },
                        "Could not send show Terrain!",
                    );
                };
            for guest_id in guest_ids {
                send_terrain_update(*guest_id, terrain_update.clone());
            }
            terrain_update.world_id = Some(world_id.clone());
            for admin_id in admin_ids {
                send_terrain_update(*admin_id, terrain_update.clone());
            }
        }
    }

    pub fn actor_reconnected(&mut self, actor_id: &ActorId) {
        if let Some(guest) = self.guests.get_mut(actor_id) {
            guest.guest_com.connected = true;
        }
        if let Some(admin) = self.admins.get_mut(actor_id) {
            admin.connected = true;
        }
    }

    fn handle_guest_events(&mut self, module: &Module) {
        for event in self
            .module_communication
            .input_receiver
            .guest_to_module_receiver
            .drain()
        {
            let GuestToModule {
                guest_id,
                event_type,
            } = event;
            match event_type.event_type {
                GuestToModuleEvent::ControlInput(input) => {
                    Self::set_guest_input(&mut self.guests, &guest_id, input)
                }
                GuestToModuleEvent::GameSetupDone => {
                    Self::set_resources_loaded(&mut self.guests, &guest_id);
                    if let Some(world_id) = self.guest_to_world.get(&guest_id) {
                        Self::send_initial_world_events(
                            &mut self
                                .module_communication
                                .output_sender
                                .game_system_to_guest_sender,
                            &self.world_map,
                            self.instance_id.clone(),
                            guest_id,
                            world_id,
                            module.id.clone(),
                            false,
                        );
                    }
                }
                GuestToModuleEvent::WantToChangeModule(_exit_slot) => {
                    debug!("WantToChangeModule not implemented!");
                }
            }
        }
    }

    pub fn send_initial_world_events_admin(
        &mut self,
        admin_id: ActorId,
        world_id: &WorldId,
        module_id: ModuleId,
    ) {
        Self::send_initial_world_events(
            &mut self
                .module_communication
                .output_sender
                .game_system_to_guest_sender,
            &self.world_map,
            self.instance_id.clone(),
            admin_id,
            world_id,
            module_id,
            true,
        );
    }

    pub fn send_initial_world_events(
        sender: &mut Sender<GameSystemToGuest>,
        world_map: &HashMap<WorldId, World>,
        instance_id: GameInstanceId,
        actor_id: ActorId,
        world_id: &WorldId,
        module_id: ModuleId,
        set_world: bool,
    ) {
        if let Some(initial_terrain_event) = Self::get_initial_terrain_event(
            world_map,
            module_id.clone(),
            instance_id.clone(),
            world_id,
            set_world,
        ) {
            send_and_log_error(
                sender,
                GuestEvent {
                    guest_id: actor_id,
                    event_type: initial_terrain_event,
                },
            );
        }
        if let Some(world) = world_map.get(world_id) {
            send_and_log_error(
                sender,
                GuestEvent {
                    guest_id: actor_id,
                    event_type: ModuleInstanceEvent {
                        module_id,
                        instance_id,
                        world_id: if set_world {
                            Some(world_id.clone())
                        } else {
                            None
                        },
                        event_type: GameSystemToGuestEvent::ShowScene(world.world_scene.clone()),
                    }});
        }
    }

    pub fn get_initial_terrain_event(
        world_map: &HashMap<WorldId, World>,
        module_id: ModuleId,
        instance_id: GameInstanceId,
        world_id: &WorldId,
        set_world: bool,
    ) -> Option<ModuleInstanceEvent<GameSystemToGuestEvent>> {
        Self::get_all_terrain(world_map, world_id).map(|terrain| ModuleInstanceEvent {
            module_id,
            instance_id,
            world_id: if set_world {
                Some(world_id.clone())
            } else {
                None
            },
            event_type: GameSystemToGuestEvent::ShowTerrain(terrain),
        })
    }

    pub fn let_admin_enter(
        &mut self,
        admin: &Admin,
        world_id: WorldId,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.world_to_admin.insert_entry(&world_id, admin.id);
        self.admin_to_world
            .insert_entry(&admin.id, world_id.clone());
        self.admins.entry(admin.id).or_insert(ModuleAdmin {
            id: admin.id,
            connected: false,
            resources_loaded: false,
        });

        Ok(EnterSuccessState::Entered)
    }

    pub fn let_admin_leave(
        &mut self,
        admin: &Admin,
        world_id: WorldId,
    ) -> Result<LeaveSuccessState, LeaveFailedState> {
        self.admin_to_world.remove_entry(&admin.id, &world_id);
        self.world_to_admin.remove_entry(&world_id, &admin.id);
        if self.admin_to_world.len(&admin.id) == 0 {
            self.admins.remove(&admin.id);
            self.admin_to_world.remove(&admin.id);
        }
        Ok(LeaveSuccessState::Left)
    }

    pub fn try_enter(
        &mut self,
        guest: &Guest,
        _module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.guests.insert(
            guest.id,
            ModuleGuest {
                id: guest.id,
                guest_com: GuestCommunication {
                    connected: true,
                    resources_loaded: false,
                },
                guest_input: GuestInput::new(),
                last_input_time: Instant::now(),
                world_id: None,
            },
        );

        Ok(EnterSuccessState::Entered)
    }

    pub fn try_leave(&mut self, _guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        Ok(LeaveSuccessState::Left)
    }

    pub fn get_all_terrain(
        world_map: &HashMap<WorldId, World>,
        world_id: &WorldId,
    ) -> Option<Vec<(LayerKind, Vec<Chunk>)>> {
        world_map.get(world_id).map(|world| {
            world
                .terrain
                .iter()
                .map(|(a, b)| (a.clone(), b.values().cloned().collect()))
                .collect()
        })
    }
}
