use std::collections::{HashMap, HashSet};

use apecs::World;
use log::{debug, error};
use tokio::time::Instant;

use crate::core::blueprint::Module;
use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication_input, EnterFailedState, EnterSuccessState, GameSystemToGuestEvent,
    GuestEvent, GuestToModule, LeaveFailedState, LeaveSuccessState, ModuleInputSender,
    ModuleInstanceEvent, ModuleOutputSender,
};
use crate::core::module::{GuestInput, GuestToModuleEvent, SystemToModuleEvent};
use crate::core::module_system::def::{DynamicGameModule, GuestMap, ModuleCommunication};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::resource_module::def::GuestId;

impl DynamicGameModule {
    pub fn create(
        blueprint: Module,
        instance_id: GameInstanceId,
        module_output_sender: ModuleOutputSender,
    ) -> (DynamicGameModule, ModuleInputSender) {
        let (module_input_sender, module_input_receiver) = create_module_communication_input();
        let dynamic_module = DynamicGameModule {
            world: World::default(),
            blueprint,
            guests: HashMap::new(),
            admins: HashSet::new(),
            module_communication: ModuleCommunication::new(
                module_input_receiver,
                module_output_sender,
            ),
            instance_id,
        };
        (dynamic_module, module_input_sender)
    }

    pub fn name(&self) -> String {
        self.blueprint.name.clone()
    }

    fn set_guest_input(guests: &mut GuestMap, guest_id: &GuestId, input: GuestInput) {
        if let Some(guest) = guests.get_mut(guest_id) {
            guest.guest_input = input;
            guest.last_input_time = Instant::now();
        }
    }

    pub fn update(&mut self) {
        self.handle_guest_events();
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
                SystemToModuleEvent::Disconnected(guest_id) => {
                    if let Some(guest) = self.guests.get_mut(&guest_id) {
                        guest.guest_com.connected = false;
                    }
                }
                SystemToModuleEvent::Reconnected(guest_id) => {
                    if let Some(guest) = self.guests.get_mut(&guest_id) {
                        guest.guest_com.connected = true;
                    }
                }
            }
        }
    }

    fn handle_guest_events(&mut self) {
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
                    if let Err(err) = self
                        .module_communication
                        .output_sender
                        .game_system_to_guest_sender
                        .send(GuestEvent {
                            guest_id,
                            event_type: {
                                ModuleInstanceEvent {
                                    module_name: self.blueprint.name.clone(),
                                    instance_id: self.instance_id.clone(),
                                    event_type: GameSystemToGuestEvent::OpenMenu(
                                        "login-menu".into(),
                                    ),
                                }
                            },
                        })
                    {
                        error!("{:?}", err);
                    }
                }
                GuestToModuleEvent::WantToChangeModule(_exit_slot) => {
                    debug!("WantToChangeModule not implemented!");
                }
            }
        }
    }
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
