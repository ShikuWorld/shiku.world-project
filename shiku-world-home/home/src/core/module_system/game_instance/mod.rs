use std::collections::HashMap;

use log::error;
use rapier2d::prelude::Real;
use snowflake::SnowflakeIdBucket;

use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    EnterFailedState, EnterSuccessState, LeaveFailedState, LeaveSuccessState, ModuleInputReceiver,
    ModuleInputSender, ModuleOutputReceiver, ModuleOutputSender,
};
use crate::core::module_system::def::DynamicGameModule;
use crate::core::module_system::error::CreateModuleError;
use crate::core::{blueprint, Snowflake, TARGET_FRAME_DURATION};
use crate::resource_module::def::{GuestId, ResourceModule};

pub type GameInstanceId = Snowflake;

pub struct GameInstanceManager {
    pub(crate) game_instances: HashMap<GameInstanceId, GameInstance>,
    pub(crate) inactive_game_instances: Vec<GameInstanceId>,
    pub(crate) guest_to_game_instance_map: HashMap<GuestId, GameInstanceId>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
    pub(crate) module_blueprint: blueprint::Module,
    pub(crate) game_instance_timeout: Real,
    pub(crate) max_guest_count_per_instance: usize,
    pub(crate) min_guest_count_per_instance: usize,
    pub(crate) close_after_full: bool,
    pub(crate) instance_id_gen: SnowflakeIdBucket,
}

impl GameInstanceManager {
    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
        min_guest_count_per_instance: usize,
        max_guest_count_per_instance: usize,
        close_after_full: bool,
        module_blueprint: blueprint::Module,
    ) -> GameInstanceManager {
        GameInstanceManager {
            game_instances: HashMap::new(),
            inactive_game_instances: Vec::new(),
            guest_to_game_instance_map: HashMap::new(),
            instance_id_gen: SnowflakeIdBucket::new(2, 20),
            game_instance_timeout: 30000.0,
            close_after_full,
            input_receiver,
            output_sender,
            module_blueprint,
            min_guest_count_per_instance,
            max_guest_count_per_instance,
        }
    }

    pub fn update(&mut self) {
        self.relay_messages_to_correct_instances();

        for game_instance in self.game_instances.values_mut() {
            game_instance.update();
            if game_instance.dynamic_module.guests.len() > 0 {
                game_instance.inactive_time = 0.0;
            }
            game_instance.inactive_time += TARGET_FRAME_DURATION;
            if game_instance.inactive_time > self.game_instance_timeout {
                self.inactive_game_instances.push(game_instance.id.clone());
            }
        }

        for inactive_game_instanced_id in self.inactive_game_instances.drain(..) {
            self.game_instances.remove(&inactive_game_instanced_id);
        }
    }

    pub fn try_enter(
        &mut self,
        guest: &Guest,
        module_enter_slot: &ModuleEnterSlot,
        resource_module: &mut ResourceModule,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        if self.guest_to_game_instance_map.contains_key(&guest.id) {
            return Err(EnterFailedState::AlreadyEntered);
        }

        let game_instance_id = self.lazy_get_game_instance_for_guest_to_join(resource_module)?;
        if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
            let enter_state = game_instance
                .dynamic_module
                .try_enter(guest, module_enter_slot);
            if enter_state.is_ok() {
                let game_instance_id = game_instance.id.clone();
                self.guest_to_game_instance_map
                    .insert(guest.id.clone(), game_instance_id);
                if self.close_after_full
                    && game_instance.dynamic_module.guests.len()
                        >= self.max_guest_count_per_instance
                {
                    game_instance.closed = true;
                }
            }

            return enter_state;
        }

        Err(EnterFailedState::GameInstanceNotFoundWTF)
    }

    pub fn try_leave(&mut self, guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        if let Some(game_instance_id) = self.guest_to_game_instance_map.remove(&guest.id) {
            if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
                return game_instance.dynamic_module.try_leave(guest);
            }
        }

        return Err(LeaveFailedState::NotInModule);
    }

    fn relay_messages_to_correct_instances(&mut self) {
        for message in self.input_receiver.guest_to_module_receiver.drain() {
            if let Some(game_instance_id) = self.guest_to_game_instance_map.get(&message.guest_id) {
                if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
                    if let Err(err) = game_instance
                        .input_sender
                        .guest_to_module_sender
                        .send(message)
                    {
                        error!(
                            "Game instance message could not send guest message to module?! {:?}",
                            err
                        );
                    }
                }
            }
        }

        for message in self.input_receiver.system_to_module_receiver.drain() {
            if let Some(game_instance_id) = self.guest_to_game_instance_map.get(&message.guest_id) {
                if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
                    if let Err(err) = game_instance
                        .input_sender
                        .system_to_module_sender
                        .send(message)
                    {
                        error!(
                            "Game instance message could not send system message to module?! {:?}",
                            err
                        );
                    }
                }
            }
        }
    }

    fn lazy_get_game_instance_for_guest_to_join(
        &mut self,
        resource_module: &mut ResourceModule,
    ) -> Result<GameInstanceId, CreateModuleError> {
        let max_guest_count = self.max_guest_count_per_instance;
        let mut game_instance_id_found = None;

        for game_instance in self.game_instances.values_mut() {
            if !game_instance.closed && game_instance.dynamic_module.guests.len() < max_guest_count
            {
                game_instance_id_found = Some(game_instance.id.clone());
                break;
            }
        }

        if let Some(game_instance_id) = game_instance_id_found {
            return Ok(game_instance_id);
        }

        let new_game_instance = GameInstance::new(
            self.instance_id_gen.get_id(),
            self.module_blueprint.clone(),
            resource_module,
        )?;
        let new_game_instance_id = new_game_instance.id.clone();
        self.game_instances
            .entry(new_game_instance.id.clone())
            .or_insert(new_game_instance);

        return Ok(new_game_instance_id);
    }
}

pub struct GameInstance {
    pub(crate) id: GameInstanceId,
    pub(crate) inactive_time: Real,
    pub(crate) dynamic_module: DynamicGameModule,
    pub(crate) output_receiver: ModuleOutputReceiver,
    pub(crate) input_sender: ModuleInputSender,
    pub(crate) closed: bool,
}

impl GameInstance {
    pub fn new(
        id: GameInstanceId,
        module_blueprint: blueprint::Module,
        resource_module: &mut ResourceModule,
    ) -> Result<GameInstance, CreateModuleError> {
        let (dynamic_module, output_receiver, input_sender) =
            DynamicGameModule::create(module_blueprint, resource_module)?;
        Ok(GameInstance {
            id,
            dynamic_module,
            output_receiver,
            input_sender,
            inactive_time: 0.0,
            closed: false,
        })
    }

    pub fn update(&mut self) {
        self.dynamic_module.update();
    }
}
