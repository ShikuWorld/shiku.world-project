use crate::argument_module::game_module::generated::ArgumentGameEntityManager;
use crate::argument_module::game_module::ArgumentGameModule;
use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity_manager::EntityManager;
use crate::core::game_instance::GameInstanceManager;
use crate::core::game_module_communication::{
    GameModuleCommunication, GameModuleCommunicationCallbacks,
};
use crate::core::resource_watcher::ResourceWatcher;
use crate::core::Snowflake;
use crate::resource_module::def::GuestId;
use crate::slime_1_module::def::Slime1SimulationConfig;
use crate::slime_1_module::game_module::generated::Slime1GameEntityManager;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ArgumentBasicGameModule =
    BasicGameModule<ArgumentGameEntityManager, ArgumentSimulationConfig>;

pub struct ArgumentModule {
    pub(crate) game_instance_manager: GameInstanceManager<
        ArgumentGameEntityManager,
        ArgumentSimulationConfig,
        ArgumentGameModule,
    >,
    pub(crate) resource_watcher: ResourceWatcher,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArgumentSimulationConfig {
    pub(crate) stick_offset_x: Real,
    pub(crate) stick_offset_y: Real,
    pub(crate) stick_gravity: Real,
    pub(crate) throw_force_x: Real,
    pub(crate) throw_force_y: Real,
    pub(crate) afk_time: u64,
}

pub struct ArgumentModuleExitSlots {
    pub exit: &'static str,
}

pub struct ArgumentModuleEnterSlots {
    pub enter: &'static str,
}
