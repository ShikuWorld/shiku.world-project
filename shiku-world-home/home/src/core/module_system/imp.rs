use std::collections::HashMap;

use apecs::World;
use log::{debug, error};
use tokio::time::Instant;

use crate::core::blueprint::def::{BlueprintService, GameMap, Module};
use crate::core::guest::{Admin, Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication_input, EnterFailedState, EnterSuccessState, GameSystemToGuestEvent,
    GuestEvent, GuestToModule, LeaveFailedState, LeaveSuccessState, ModuleInputSender,
    ModuleInstanceEvent, ModuleOutputSender,
};
use crate::core::module::{GuestInput, GuestToModuleEvent, SystemToModuleEvent};
use crate::core::module_system::def::{
    DynamicGameModule, GuestCommunication, GuestMap, ModuleAdmin, ModuleCommunication, ModuleGuest,
    WorldId,
};
use crate::core::module_system::error::{CreateWorldError, DestroyWorldError};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::{send_and_log_error, LazyHashmapSet};
use crate::resource_module::def::ActorId;

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
            instance_id,
        };
        let game_maps = match BlueprintService::load_all_maps_for_module(module) {
            Ok(maps) => maps,
            Err(err) => {
                error!("Could not load maps for module to create worlds {:?}", err);
                Vec::new()
            }
        };
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
        self.world_map
            .insert(game_map.world_id.clone(), World::default());
        self.world_to_admin.init(game_map.world_id.clone());
        self.world_to_guest.init(game_map.world_id.clone());
        Ok(game_map.world_id.clone())
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
        self.handle_system_events();
    }

    fn handle_system_events(&mut self) {
        for event in self
            .module_communication
            .input_receiver
            .system_to_module_receiver
            .drain()
        {
            match event.event_type {
                SystemToModuleEvent::Disconnected(actor_id) => {
                    if let Some(guest) = self.guests.get_mut(&actor_id) {
                        guest.guest_com.connected = false;
                    }
                    if let Some(admin) = self.admins.get_mut(&actor_id) {
                        admin.connected = false;
                    }
                }
                SystemToModuleEvent::Reconnected(actor_id) => {
                    if let Some(guest) = self.guests.get_mut(&actor_id) {
                        guest.guest_com.connected = true;
                    }
                    if let Some(admin) = self.admins.get_mut(&actor_id) {
                        admin.connected = true;
                    }
                }
            }
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
                    send_and_log_error(
                        &mut self
                            .module_communication
                            .output_sender
                            .game_system_to_guest_sender,
                        GuestEvent {
                            guest_id,
                            event_type: {
                                ModuleInstanceEvent {
                                    module_id: module.id.clone(),
                                    instance_id: self.instance_id.clone(),
                                    world_id: None,
                                    event_type: GameSystemToGuestEvent::OpenMenu(
                                        "login-menu".into(),
                                    ),
                                }
                            },
                        },
                    );
                }
                GuestToModuleEvent::WantToChangeModule(_exit_slot) => {
                    debug!("WantToChangeModule not implemented!");
                }
            }
        }
    }

    pub fn let_admin_enter(
        &mut self,
        admin: &Admin,
        world_id: WorldId,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.world_to_admin.insert_entry(&world_id, admin.id);
        self.admin_to_world.insert_entry(&admin.id, world_id);
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
}
