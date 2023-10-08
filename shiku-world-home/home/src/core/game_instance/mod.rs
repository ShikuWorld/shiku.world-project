use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity_manager::EntityManager;
use crate::core::game_module_communication::{
    GameModuleCommunication, GameModuleCommunicationCallbacks,
};
use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication_input, EnterFailedState, EnterSuccessState, GameSystemToGuestEvent,
    GuestEvent, LeaveFailedState, LeaveSuccessState, ModuleInputReceiver, ModuleInputSender,
    ModuleName, ModuleOutputSender,
};
use crate::core::{safe_unwrap_ref, Snowflake, TARGET_FRAME_DURATION};
use crate::resource_module::def::GuestId;
use log::{debug, error};
use rapier2d::prelude::Real;
use snowflake::SnowflakeIdBucket;
use std::collections::HashMap;

pub type GameInstanceId = Snowflake;

pub struct GameInstanceManager<
    E: EntityManager,
    SC: Clone,
    G: GameModuleCommunicationCallbacks<E, SC>,
> {
    pub(crate) game_instances: HashMap<GameInstanceId, GameInstance<E, SC, G>>,
    pub(crate) inactive_game_instances: Vec<GameInstanceId>,
    pub(crate) guest_to_game_instance_map: HashMap<GuestId, GameInstanceId>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
    pub(crate) map_path: String,
    pub(crate) module_name: String,
    pub(crate) game_instance_timeout: Real,
    pub(crate) max_guest_count_per_instance: usize,
    pub(crate) min_guest_count_per_instance: usize,
    pub(crate) close_after_full: bool,
    pub(crate) simulation_config: SC,
    pub(crate) entity_manager_factory: fn() -> E,
    pub(crate) game_module_factory: fn(&mut BasicGameModule<E, SC>) -> G,
    pub(crate) instance_id_gen: SnowflakeIdBucket,
}

impl<E: EntityManager, SC: Clone, G: GameModuleCommunicationCallbacks<E, SC>>
    GameInstanceManager<E, SC, G>
{
    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
        min_guest_count_per_instance: usize,
        max_guest_count_per_instance: usize,
        close_after_full: bool,
        simulation_config: SC,
        map_path: String,
        module_name: String,
        entity_manager_factory: fn() -> E,
        game_module_factory: fn(&mut BasicGameModule<E, SC>) -> G,
    ) -> GameInstanceManager<E, SC, G> {
        GameInstanceManager {
            game_instances: HashMap::new(),
            inactive_game_instances: Vec::new(),
            guest_to_game_instance_map: HashMap::new(),
            instance_id_gen: SnowflakeIdBucket::new(2, 20),
            game_instance_timeout: 30000.0,
            close_after_full,
            input_receiver,
            output_sender,
            map_path,
            module_name,
            simulation_config,
            entity_manager_factory,
            game_module_factory,
            min_guest_count_per_instance,
            max_guest_count_per_instance,
        }
    }

    pub fn update(&mut self) {
        self.relay_messages_to_correct_instances();

        for game_instance in self.game_instances.values_mut() {
            game_instance.update();
            if game_instance.game_module_communication.current_guests.len() > 0 {
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
    ) -> Result<EnterSuccessState, EnterFailedState> {
        if self.guest_to_game_instance_map.contains_key(&guest.id) {
            return Err(EnterFailedState::AlreadyEntered);
        }

        let game_instance_id = self.lazy_get_game_instance_for_guest_to_join();
        if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
            let enter_state = game_instance.try_enter(guest, module_enter_slot);
            if enter_state.is_ok() {
                let game_instance_id = game_instance.id.clone();
                self.guest_to_game_instance_map
                    .insert(guest.id.clone(), game_instance_id);
                if self.close_after_full
                    && game_instance.game_module_communication.current_guests.len()
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
                return game_instance.try_leave(guest);
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

    fn create_new_game_instance(&mut self) -> GameInstance<E, SC, G> {
        let (module_input_sender, module_input_receiver) = create_module_communication_input();

        let entity_manager = (self.entity_manager_factory)();
        let mut basic_game_module = BasicGameModule::new(
            entity_manager,
            self.simulation_config.clone(),
            self.map_path.as_str(),
        );
        let game_module = (self.game_module_factory)(&mut basic_game_module);

        GameInstance::new(
            self.instance_id_gen.get_id(),
            self.module_name.clone(),
            module_input_sender,
            module_input_receiver,
            self.output_sender.clone(),
            self.close_after_full,
            game_module,
            basic_game_module,
        )
    }

    fn lazy_get_game_instance_for_guest_to_join(&mut self) -> GameInstanceId {
        let max_guest_count = self.max_guest_count_per_instance;
        let mut game_instance_id_found = None;

        for game_instance in self.game_instances.values_mut() {
            if !game_instance.closed
                && game_instance.game_module_communication.current_guests.len() < max_guest_count
            {
                game_instance_id_found = Some(game_instance.id.clone());
                break;
            }
        }

        if let Some(game_instance_id) = game_instance_id_found {
            return game_instance_id;
        }

        let new_game_instance = self.create_new_game_instance();
        let new_game_instance_id = new_game_instance.id.clone();
        self.game_instances
            .entry(new_game_instance.id.clone())
            .or_insert(new_game_instance);

        return new_game_instance_id;
    }
}

pub struct GameInstance<E: EntityManager, SC, G: GameModuleCommunicationCallbacks<E, SC>> {
    pub(crate) id: GameInstanceId,
    pub(crate) game_module_communication: GameModuleCommunication,
    pub(crate) inactive_time: Real,
    pub(crate) input_sender: ModuleInputSender,
    pub(crate) basic_game_module: BasicGameModule<E, SC>,
    pub(crate) game_module: G,
    pub(crate) module_name: ModuleName,
    pub(crate) closed: bool,
}

impl<E: EntityManager, SC: Clone, G: GameModuleCommunicationCallbacks<E, SC>>
    GameInstance<E, SC, G>
{
    pub fn new(
        id: GameInstanceId,
        module_name: ModuleName,
        input_sender: ModuleInputSender,
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
        close_after_full: bool,
        game_module: G,
        basic_game_module: BasicGameModule<E, SC>,
    ) -> GameInstance<E, SC, G> {
        GameInstance {
            id,
            module_name,
            input_sender,
            inactive_time: 0.0,
            basic_game_module,
            game_module,
            closed: false,
            game_module_communication: GameModuleCommunication::new(input_receiver, output_sender),
        }
    }

    pub fn update(&mut self) {
        self.game_module_communication
            .process_system_input_events(&mut self.game_module, &mut self.basic_game_module);
        if let Err(err) = GameModuleCommunication::process_guest_input_events(
            &mut self.game_module_communication,
            &mut self.game_module,
            &mut self.basic_game_module,
        ) {
            error!("{:?}", err);
        }

        self.game_module.update(
            &mut self.basic_game_module,
            &mut self.game_module_communication.output_sender,
        );
        self.basic_game_module.update();

        GameModuleCommunication::send_entity_updates_to_guests(
            &mut self.game_module_communication.output_sender,
            &self.game_module_communication.current_guests,
            &mut self.basic_game_module,
            self.module_name.clone(),
        );
    }

    pub fn try_enter(
        &mut self,
        guest: &Guest,
        module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.game_module_communication.guest_enter(
            &guest.id,
            module_enter_slot,
            safe_unwrap_ref(
                &guest.persisted_guest,
                EnterFailedState::PersistedStateGoneMissingGoneWild,
            )?,
            &mut self.game_module,
            &mut self.basic_game_module,
        );

        debug!("Guest entered instance of {}.", self.module_name);

        Ok(EnterSuccessState::Entered)
    }

    pub fn try_leave(&mut self, guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        self.game_module_communication.guest_leave(
            &guest.id,
            safe_unwrap_ref(
                &guest.persisted_guest,
                LeaveFailedState::PersistedStateGoneMissingGoneWild,
            )?,
            &mut self.game_module,
            &mut self.basic_game_module,
        );
        debug!("Guest left instance of {}.", self.module_name);

        if let Err(err) = self
            .game_module_communication
            .output_sender
            .game_system_to_guest_sender
            .send(GuestEvent {
                guest_id: guest.id.clone(),
                event_type: GameSystemToGuestEvent::RemoveAllEntities(self.module_name.clone()),
            })
        {
            error!("Could not send remove all entities to guest, what?!");
        }

        Ok(LeaveSuccessState::Left)
    }
}
