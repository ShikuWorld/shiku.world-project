use crate::core::blueprint;
use crate::core::blueprint::{BlueprintError, BlueprintService};
use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    EnterFailedState, EnterSuccessState, GameSystemToGuest, LeaveFailedState, LeaveSuccessState,
};
use crate::core::module::{
    GuestEvent, GuestInput, GuestToModuleEvent, ModuleInputReceiver, ModuleName,
    ModuleOutputSender, SystemToModuleEvent,
};
use crate::core::safe_unwrap;
use crate::persistence_module::models::PersistedGuest;
use crate::resource_module::def::{GuestId, ResourceFile, ResourceModule};
use crate::resource_module::errors::ResourceParseError;
use apecs::World;
use flume::SendError;
use log::{debug, error};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
pub enum ProcessGuestInputError {
    ExpectedValueNotInMap(String),
    SendToGuestError(SendError<GameSystemToGuest>),
}

impl From<SendError<GameSystemToGuest>> for ProcessGuestInputError {
    fn from(err: SendError<GameSystemToGuest>) -> Self {
        ProcessGuestInputError::SendToGuestError(err)
    }
}

pub struct GuestCommunication {
    pub resources_loaded: bool,
    pub connected: bool,
}

pub struct ModuleCommunication {
    pub(crate) current_guests: HashMap<GuestId, GuestCommunication>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
}

impl ModuleCommunication {
    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> ModuleCommunication {
        ModuleCommunication {
            current_guests: HashMap::new(),
            input_receiver,
            output_sender,
        }
    }

    pub fn guest_enter<T: ModuleCommunicationCallbacks>(
        &mut self,
        guest_id: &GuestId,
        persisted_guest: &PersistedGuest,
        callback_entity: &mut T,
    ) {
        debug!("{} joined", persisted_guest.info.display_name);

        self.current_guests.insert(
            *guest_id,
            GuestCommunication {
                resources_loaded: false,
                connected: true,
            },
        );

        callback_entity.on_guest_enter(guest_id, persisted_guest);
    }

    pub fn guest_leave<T: ModuleCommunicationCallbacks>(
        &mut self,
        guest_id: &GuestId,
        persisted_guest: &PersistedGuest,
        callback_entity: &mut T,
    ) {
        debug!("{} left", persisted_guest.info.display_name);
        self.current_guests.remove(guest_id);

        callback_entity.on_guest_leave(guest_id, persisted_guest);
    }

    pub fn process_input_events<T: ModuleCommunicationCallbacks>(
        &mut self,
        callback_entity: &mut T,
    ) {
        if let Err(err) = self.process_guest_input_events(callback_entity) {
            error!("Could not handle guest input events! {:?}", err);
        }
        self.process_system_input_events(callback_entity);
    }

    fn process_system_input_events<T: ModuleCommunicationCallbacks>(
        &mut self,
        callback_entity: &mut T,
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
                        callback_entity.on_guest_disconnected(&guest_id);
                    } else {
                        error!("Could not get guest????");
                        return;
                    }
                }
                SystemToModuleEvent::Reconnected => {
                    if let Some(guest) = self.current_guests.get_mut(&guest_id) {
                        debug!("Guest Reconnected!");
                        guest.connected = true;
                        callback_entity.on_guest_reconnected(&guest_id);
                    } else {
                        error!("Could not get guest????");
                        return;
                    }
                }
                SystemToModuleEvent::AlreadyLoggedIn => {}
            }
        }
    }

    fn process_guest_input_events<T: ModuleCommunicationCallbacks>(
        &mut self,
        callback_entity: &mut T,
    ) -> Result<(), ProcessGuestInputError> {
        for GuestEvent {
            guest_id,
            event_type,
        } in self.input_receiver.guest_to_module_receiver.drain()
        {
            match event_type {
                GuestToModuleEvent::ResourcesLoaded(module_name) => {
                    debug!("Resources for {} finished loading for guest", module_name);

                    let mut guest = safe_unwrap(
                        self.current_guests.get_mut(&guest_id),
                        ProcessGuestInputError::ExpectedValueNotInMap(
                            "Could not get guest, but they should be in here.".to_string(),
                        ),
                    )?;

                    guest.resources_loaded = true;

                    callback_entity.on_guest_ready_to_accept_entities(&guest_id);
                }
                GuestToModuleEvent::ControlInput(input) => {
                    callback_entity.on_guest_input(&guest_id, input);
                }
                GuestToModuleEvent::WantToChangeModule(module_name) => {
                    callback_entity.on_want_to_change_module(&guest_id, module_name);
                }
            }
        }

        Ok(())
    }
}

#[allow(unused_variables)]
pub trait ModuleCommunicationCallbacks {
    fn update(&mut self, output_sender: &mut ModuleOutputSender) {}
    fn on_guest_enter(&mut self, guest_id: &GuestId, persisted_guest: &PersistedGuest) {
        debug!("on_guest_enter not implemented.");
    }
    fn on_guest_leave(&mut self, guest_id: &GuestId, persisted_guest: &PersistedGuest) {
        debug!("on_guest_leave not implemented.");
    }
    fn on_guest_disconnected(&mut self, guest_id: &GuestId) {
        debug!("on_guest_disconnected not implemented.");
    }
    fn on_guest_reconnected(&mut self, guest_id: &GuestId) {
        debug!("on_guest_reconnected not implemented.");
    }
    fn on_guest_ready_to_accept_entities(&mut self, guest_id: &GuestId) {
        debug!("on_guest_ready_to_accept_entities not implemented.");
    }
    fn on_guest_input(&mut self, guest_id: &GuestId, input: GuestInput) {
        debug!("on_guest_input not implemented.");
    }
    fn on_want_to_change_module(&mut self, guest_id: &GuestId, module_name: ModuleName) {
        debug!("on_want_to_change_module not implemented.");
    }
}

pub struct DynamicGameModule {
    pub world: World,
    pub blueprint: blueprint::Module,
    pub guests: HashMap<GuestId, ModuleGuest>,
    pub module_input_receiver: ModuleInputReceiver,
    pub module_output_sender: ModuleOutputSender,
}

pub struct ModuleGuest {
    id: GuestId,
}

pub struct ModuleService {
    available_modules: HashMap<String, DynamicGameModule>,
}

#[derive(Error, Debug)]
pub enum CreateModuleError {
    #[error("Could not create blueprint.")]
    BlueprintError(#[from] BlueprintError),
}

impl DynamicGameModule {
    pub fn create(
        module_name: String,
        module_input_receiver: ModuleInputReceiver,
        module_output_sender: ModuleOutputSender,
        blueprint_service: &BlueprintService,
    ) -> Result<DynamicGameModule, CreateModuleError> {
        let blueprint = blueprint_service.create_module(module_name)?;

        Ok(DynamicGameModule {
            world: World::default(),
            blueprint,
            guests: HashMap::new(),
            module_input_receiver,
            module_output_sender,
        })
    }

    pub fn lazy_load(
        module_name: String,
        module_input_receiver: ModuleInputReceiver,
        module_output_sender: ModuleOutputSender,
        blueprint_service: &BlueprintService,
    ) -> Result<DynamicGameModule, CreateModuleError> {
        let blueprint = blueprint_service.lazy_load_module(module_name)?;

        Ok(DynamicGameModule {
            world: World::default(),
            blueprint,
            guests: HashMap::new(),
            module_input_receiver,
            module_output_sender,
        })
    }

    pub fn name(&self) -> String {
        self.blueprint.name.clone()
    }

    fn get_base_resource_file(&self) -> ResourceFile {
        ResourceFile {
            resources: Vec::new(),
            module_name: "test".into(),
        }
    }
    fn get_resource_json(&self) -> String {
        "".into()
    }

    pub fn register_resources(
        &self,
        resource_module: &mut ResourceModule,
    ) -> Result<(), ResourceParseError> {
        resource_module.register_resources_for_module(
            self.blueprint.name.clone(),
            self.blueprint.name.clone(),
            self.get_base_resource_file(),
            Some(self.get_resource_json()),
        )?;

        Ok(())
    }
    pub fn update(&mut self) {}
    pub fn try_enter(
        &mut self,
        _guest: &Guest,
        _module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        Ok(EnterSuccessState::Entered)
    }

    pub fn try_leave(&mut self, _guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        Ok(LeaveSuccessState::Left)
    }
}

impl ModuleService {
    pub fn new() -> ModuleService {
        ModuleService {
            available_modules: HashMap::new(),
        }
    }
    pub fn add_module() {}
    pub fn remove_module() {}
    pub fn load_module() {}
    pub fn save_module() {}
}
