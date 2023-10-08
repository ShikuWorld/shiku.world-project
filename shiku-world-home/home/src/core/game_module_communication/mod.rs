use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity_manager::EntityManager;

use crate::core::entity::render::CameraSettings;
use crate::core::module::GameSystemToGuest;
use crate::core::module::{
    GameSystemToGuestEvent, GuestEvent, GuestInput, GuestToModuleEvent, ModuleInputReceiver,
    ModuleName, ModuleOutputSender, SystemToModuleEvent,
};
use crate::core::module_system::{GuestCommunication, ProcessGuestInputError};
use crate::core::safe_unwrap;
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::GuestId;
use flume::Sender;
use log::{debug, error};
use rapier2d::math::Real;
use std::collections::HashMap;

pub struct GameModuleCommunication {
    pub(crate) current_guests: HashMap<GuestId, GuestCommunication>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
}

impl GameModuleCommunication {
    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> GameModuleCommunication {
        GameModuleCommunication {
            current_guests: HashMap::new(),
            input_receiver,
            output_sender,
        }
    }

    pub fn guest_enter<E: EntityManager, T: GameModuleCommunicationCallbacks<E, S>, S>(
        &mut self,
        guest_id: &GuestId,
        entry_point_id: &String,
        persisted_guest: &PersistedGuest,
        callback_entity: &mut T,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("{} joined", persisted_guest.info.display_name);

        self.current_guests.insert(
            *guest_id,
            GuestCommunication {
                resources_loaded: false,
                connected: true,
            },
        );

        callback_entity.on_guest_enter(
            guest_id,
            entry_point_id,
            persisted_guest,
            basic_game_module,
        );
    }

    pub fn guest_leave<E: EntityManager, T: GameModuleCommunicationCallbacks<E, S>, S>(
        &mut self,
        guest_id: &GuestId,
        persisted_guest: &PersistedGuest,
        callback_entity: &mut T,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("{} left", persisted_guest.info.display_name);
        self.current_guests.remove(guest_id);

        callback_entity.on_guest_leave(guest_id, persisted_guest, basic_game_module);
    }

    pub fn process_system_input_events<
        E: EntityManager,
        T: GameModuleCommunicationCallbacks<E, S>,
        S,
    >(
        &mut self,
        callback_entity: &mut T,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        for GuestEvent {
            guest_id,
            event_type,
        } in self.input_receiver.system_to_module_receiver.drain()
        {
            match event_type {
                SystemToModuleEvent::Disconnected => {
                    debug!("Guest Disconnected!");
                    if let Some(guest) = self.current_guests.get_mut(&guest_id) {
                        guest.connected = false;
                        guest.resources_loaded = false;
                        callback_entity.on_guest_disconnected(&guest_id, basic_game_module);
                    } else {
                        error!("Could not get guest????");
                        return;
                    }
                }
                SystemToModuleEvent::Reconnected => {
                    if let Some(guest) = self.current_guests.get_mut(&guest_id) {
                        debug!("Guest Reconnected!");
                        guest.connected = true;
                        callback_entity.on_guest_reconnected(&guest_id, basic_game_module);
                    } else {
                        error!("Could not get guest????");
                        return;
                    }
                }
                SystemToModuleEvent::AlreadyLoggedIn => {}
            }
        }
    }

    pub fn process_guest_input_events<
        E: EntityManager,
        T: GameModuleCommunicationCallbacks<E, S>,
        S,
    >(
        game_module_communication: &mut GameModuleCommunication,
        callback_entity: &mut T,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) -> Result<(), ProcessGuestInputError> {
        for GuestEvent {
            guest_id,
            event_type,
        } in game_module_communication
            .input_receiver
            .guest_to_module_receiver
            .drain()
        {
            match event_type {
                GuestToModuleEvent::ResourcesLoaded(module_name) => {
                    debug!("Resources for {} finished loading for guest", module_name);

                    let mut guest = safe_unwrap(
                        game_module_communication.current_guests.get_mut(&guest_id),
                        ProcessGuestInputError::ExpectedValueNotInMap(
                            "Could not get guest, but they should be in here.".to_string(),
                        ),
                    )?;

                    guest.resources_loaded = true;

                    callback_entity.on_guest_ready_to_accept_entities(&guest_id, basic_game_module);
                    Self::send_initial_entities_to_guest(
                        &mut game_module_communication
                            .output_sender
                            .game_system_to_guest_sender,
                        &guest_id,
                        basic_game_module,
                        module_name,
                    );
                }
                GuestToModuleEvent::ControlInput(input) => {
                    callback_entity.on_guest_input(&guest_id, input);
                }
                GuestToModuleEvent::WantToChangeModule(module_name) => {
                    callback_entity.on_want_to_change_module(&guest_id, module_name);
                }
                GuestToModuleEvent::ProviderLoggedIn(_) => {
                    error!("Guest {} tried to send login token in, wtf?", guest_id);
                }
                GuestToModuleEvent::Ping => (),
            }
        }

        Ok(())
    }

    pub fn send_entity_updates_to_guests<E: EntityManager, S>(
        module_output_sender: &mut ModuleOutputSender,
        current_guests: &HashMap<GuestId, GuestCommunication>,
        basic_game_module: &mut BasicGameModule<E, S>,
        module_name: ModuleName,
    ) {
        let show_entities = basic_game_module
            .game_entity_manager
            .drain_new_show_entities();

        let remove_entities = basic_game_module
            .game_entity_manager
            .drain_new_remove_entities();

        let update_entities = basic_game_module
            .game_entity_manager
            .get_all_entity_updates();

        let position_updates = basic_game_module
            .game_entity_manager
            .get_all_entity_position_updates();

        let show_effects = basic_game_module
            .game_entity_manager
            .drain_new_show_effects();

        for (guest_id, guest_communication) in current_guests {
            if !guest_communication.connected || !guest_communication.resources_loaded {
                continue;
            }

            if !show_entities.is_empty() {
                if let Err(err) =
                    module_output_sender
                        .game_system_to_guest_sender
                        .send(GuestEvent {
                            guest_id: *guest_id,
                            event_type: GameSystemToGuestEvent::ShowEntities(
                                show_entities.clone(),
                                module_name.clone(),
                            ),
                        })
                {
                    error!("{:?}", err);
                }
            }

            if !remove_entities.is_empty() {
                if let Err(err) =
                    module_output_sender
                        .game_system_to_guest_sender
                        .send(GuestEvent {
                            guest_id: *guest_id,
                            event_type: GameSystemToGuestEvent::RemoveEntities(
                                remove_entities.clone(),
                                module_name.clone(),
                            ),
                        })
                {
                    error!("{:?}", err);
                }
            }

            if !show_effects.is_empty() {
                if let Err(err) =
                    module_output_sender
                        .game_system_to_guest_sender
                        .send(GuestEvent {
                            guest_id: *guest_id,
                            event_type: GameSystemToGuestEvent::ShowEffects(
                                show_effects.clone(),
                                module_name.clone(),
                            ),
                        })
                {
                    error!("{:?}", err);
                }
            }

            if !update_entities.is_empty() {
                if let Err(err) =
                    module_output_sender
                        .game_system_to_guest_sender
                        .send(GuestEvent {
                            guest_id: *guest_id,
                            event_type: GameSystemToGuestEvent::UpdateEntities(
                                update_entities.clone(),
                                module_name.clone(),
                            ),
                        })
                {
                    error!("{:?}", err);
                }
            }

            if !position_updates.is_empty() {
                if let Err(err) = module_output_sender.position_sender.send(GuestEvent {
                    guest_id: *guest_id,
                    event_type: position_updates.clone(),
                }) {
                    error!("{:?}", err);
                }
            }
        }
    }

    fn send_initial_entities_to_guest<E: EntityManager, S>(
        game_system_to_guest_sender: &mut Sender<GameSystemToGuest>,
        guest_id: &GuestId,
        basic_game_module: &mut BasicGameModule<E, S>,
        module_name: ModuleName,
    ) {
        let show_entities = basic_game_module
            .game_entity_manager
            .get_all_show_entities();

        if let Err(err) = game_system_to_guest_sender.send(GuestEvent {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::ShowEntities(show_entities, module_name.clone()),
        }) {
            error!("{:?}", err);
        }

        let terrain_chunks = basic_game_module
            .game_entity_manager
            .get_all_terrain_chunks();

        let tile_size = match &basic_game_module.tiled_map {
            Some(map) => map.tile_height as Real,
            None => 1.0,
        };

        let mut parallax_layers = Vec::new();
        match &basic_game_module.tiled_map {
            Some(map) => {
                for layer in &map.layers {
                    parallax_layers.push((layer.name.clone(), layer.parallax));
                }
            }
            None => (),
        };

        if let Err(err) = game_system_to_guest_sender.send(GuestEvent {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::ShowTerrainChunks(
                tile_size,
                terrain_chunks,
                module_name.clone(),
            ),
        }) {
            error!("{:?}", err);
        }

        if let Err(err) = game_system_to_guest_sender.send(GuestEvent {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::SetCamera(
                basic_game_module
                    .game_entity_manager
                    .get_current_camera_entity_for_guest(guest_id),
                module_name,
                basic_game_module
                    .base_camera_settings
                    .clone()
                    .unwrap_or_else(|| CameraSettings::default()),
            ),
        }) {
            error!("{:?}", err);
        }

        if let Err(err) = game_system_to_guest_sender.send(GuestEvent {
            guest_id: *guest_id,
            event_type: GameSystemToGuestEvent::SetParallax(parallax_layers),
        }) {
            error!("{:?}", err);
        }
    }
}

#[allow(unused_variables)]
pub trait GameModuleCommunicationCallbacks<E: EntityManager, S> {
    fn update(
        &mut self,
        basic_game_module: &mut BasicGameModule<E, S>,
        output_sender: &mut ModuleOutputSender,
    ) {
        debug!("update not implemented.");
    }
    fn on_guest_enter(
        &mut self,
        guest_id: &GuestId,
        entry_point_id: &String,
        persisted_guest: &PersistedGuest,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("on_guest_enter not implemented.");
    }
    fn on_guest_leave(
        &mut self,
        guest_id: &GuestId,
        persisted_guest: &PersistedGuest,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("on_guest_leave not implemented.");
    }
    fn on_guest_disconnected(
        &mut self,
        guest_id: &GuestId,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("on_guest_disconnected not implemented.");
    }
    fn on_guest_reconnected(
        &mut self,
        guest_id: &GuestId,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("on_guest_reconnected not implemented.");
    }
    fn on_guest_ready_to_accept_entities(
        &mut self,
        guest_id: &GuestId,
        basic_game_module: &mut BasicGameModule<E, S>,
    ) {
        debug!("on_guest_ready_to_accept_entities not implemented.");
    }
    fn on_guest_input(&mut self, guest_id: &GuestId, input: GuestInput) {
        debug!("on_guest_input not implemented.");
    }
    fn on_want_to_change_module(&mut self, guest_id: &GuestId, module_name: ModuleName) {
        debug!("on_want_to_change_module not implemented.");
    }
}
