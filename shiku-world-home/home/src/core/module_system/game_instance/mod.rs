use std::collections::HashMap;

use log::error;
use rapier2d::prelude::Real;
use snowflake::SnowflakeIdBucket;
use thiserror::Error;

use crate::core::blueprint::BlueprintError;
use crate::core::blueprint::BlueprintService;
use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication, EnterFailedState, EnterSuccessState, LeaveFailedState,
    LeaveSuccessState, ModuleInputReceiver, ModuleInputSender, ModuleOutputReceiver,
    ModuleOutputSender,
};
use crate::core::module_system::def::DynamicGameModule;
use crate::core::{blueprint, Snowflake, TARGET_FRAME_DURATION};
use crate::resource_module::def::{ActorId, ResourceFile, ResourceModule};
use crate::resource_module::errors::ResourceParseError;

#[derive(Error, Debug)]
pub enum CreateInstanceManagerError {
    #[error(transparent)]
    BlueprintError(#[from] BlueprintError),
    #[error(transparent)]
    ResourceParse(#[from] ResourceParseError),
}

pub type GameInstanceId = String;

pub struct GameInstanceManager {
    pub(crate) game_instances: HashMap<GameInstanceId, GameInstance>,
    pub(crate) inactive_game_instances: Vec<GameInstanceId>,
    pub(crate) guest_to_game_instance_map: HashMap<ActorId, GameInstanceId>,
    pub(crate) active_admin_instances: HashMap<Snowflake, Vec<GameInstanceId>>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
    pub(crate) module_blueprint: blueprint::Module,
    pub(crate) game_instance_timeout: Real,
    pub(crate) instance_id_gen: SnowflakeIdBucket,
}

impl GameInstanceManager {
    pub fn new(
        module_name: String,
        blueprint_service: &BlueprintService,
        resource_module: &mut ResourceModule,
    ) -> Result<
        (GameInstanceManager, ModuleInputSender, ModuleOutputReceiver),
        CreateInstanceManagerError,
    > {
        let module_blueprint = blueprint_service.lazy_load_module(module_name)?;
        let (input_sender, input_receiver, output_sender, output_receiver) =
            create_module_communication();
        let manager = GameInstanceManager {
            game_instances: HashMap::new(),
            inactive_game_instances: Vec::new(),
            guest_to_game_instance_map: HashMap::new(),
            active_admin_instances: HashMap::new(),
            instance_id_gen: SnowflakeIdBucket::new(1, 6),
            game_instance_timeout: 30000.0,
            input_receiver,
            output_sender,
            module_blueprint,
        };

        manager.register_resources(resource_module)?;

        Ok((manager, input_sender, output_receiver))
    }

    pub fn update(&mut self) {
        self.relay_messages_to_correct_instances();

        for game_instance in self.game_instances.values_mut() {
            game_instance.update();
            if !game_instance.dynamic_module.guests.is_empty() {
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
    ) -> Result<(GameInstanceId, EnterSuccessState), EnterFailedState> {
        if self.guest_to_game_instance_map.contains_key(&guest.id) {
            return Err(EnterFailedState::AlreadyEntered);
        }

        let game_instance_id = self.lazy_get_game_instance_for_guest_to_join();
        if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
            return match game_instance
                .dynamic_module
                .try_enter(guest, module_enter_slot)
            {
                Ok(success_state) => {
                    let game_instance_id = game_instance.id.clone();
                    self.guest_to_game_instance_map
                        .insert(guest.id, game_instance_id.clone());
                    if self.module_blueprint.close_after_full
                        && game_instance.dynamic_module.guests.len()
                            >= self.module_blueprint.max_guests
                    {
                        game_instance.closed = true;
                    }
                    Ok((game_instance_id, success_state))
                }
                Err(fail_state) => Err(fail_state),
            };
        }

        Err(EnterFailedState::GameInstanceNotFoundWTF)
    }

    pub fn try_leave(
        &mut self,
        guest: &Guest,
    ) -> Result<(GameInstanceId, LeaveSuccessState), LeaveFailedState> {
        if let Some(game_instance_id) = self.guest_to_game_instance_map.remove(&guest.id) {
            if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
                return match game_instance.dynamic_module.try_leave(guest) {
                    Ok(success_state) => Ok((game_instance_id, success_state)),
                    Err(err) => Err(err),
                };
            }
        }

        Err(LeaveFailedState::NotInModule)
    }

    fn get_base_resource_file(&self) -> ResourceFile {
        ResourceFile {
            resources: Vec::new(),
            module_name: self.module_blueprint.name.clone(),
        }
    }
    fn get_resource_json(&self) -> String {
        format!(
            "{{\"module_name\": \"{}\", \"resources\": [{{\"kind\": \"Image\", \"meta_name\": \"test\", \"path\": \"test.png\"}}]}}",
            self.module_blueprint.name
        )
    }

    pub fn register_resources(
        &self,
        resource_module: &mut ResourceModule,
    ) -> Result<(), ResourceParseError> {
        resource_module.register_resources_for_module(
            self.module_blueprint.name.clone(),
            self.module_blueprint.name.clone(),
            self.get_base_resource_file(),
            Some(self.get_resource_json()),
        )?;

        Ok(())
    }

    fn relay_messages_to_correct_instances(&mut self) {
        for message in self.input_receiver.guest_to_module_receiver.drain() {
            if let Some(game_instance) =
                self.game_instances.get_mut(&message.event_type.instance_id)
            {
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

        for message in self.input_receiver.system_to_module_receiver.drain() {
            if let Some(game_instance) = self.game_instances.get_mut(&message.instance_id) {
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

    fn lazy_get_game_instance_for_guest_to_join(&mut self) -> GameInstanceId {
        let max_guest_count = self.module_blueprint.max_guests;
        let mut game_instance_id_found = None;

        for game_instance in self.game_instances.values_mut() {
            if !game_instance.closed && game_instance.dynamic_module.guests.len() < max_guest_count
            {
                game_instance_id_found = Some(game_instance.id.clone());
                break;
            }
        }

        if let Some(game_instance_id) = game_instance_id_found {
            return game_instance_id;
        }

        let new_game_instance = GameInstance::new(
            self.instance_id_gen.get_id().to_string(),
            self.module_blueprint.clone(),
            self.output_sender.clone(),
        );
        let new_game_instance_id = new_game_instance.id.clone();
        self.game_instances
            .entry(new_game_instance.id.clone())
            .or_insert(new_game_instance);

        new_game_instance_id
    }
}

pub struct GameInstance {
    pub(crate) id: GameInstanceId,
    pub(crate) inactive_time: Real,
    pub(crate) dynamic_module: DynamicGameModule,
    pub(crate) input_sender: ModuleInputSender,
    pub(crate) closed: bool,
}

impl GameInstance {
    pub fn new(
        id: GameInstanceId,
        module_blueprint: blueprint::Module,
        output_sender: ModuleOutputSender,
    ) -> GameInstance {
        let (dynamic_module, input_sender) =
            DynamicGameModule::create(module_blueprint, id.clone(), output_sender);
        GameInstance {
            id,
            dynamic_module,
            input_sender,
            inactive_time: 0.0,
            closed: false,
        }
    }

    pub fn update(&mut self) {
        self.dynamic_module.update();
    }
}
