use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity_manager::EntityManager;
use crate::core::game_instance::GameInstanceManager;
use crate::core::game_module_communication::{
    GameModuleCommunication, GameModuleCommunicationCallbacks,
};
use crate::core::Snowflake;
use crate::lobby_module::game_module::generated::LobbyGameEntityManager;
use crate::lobby_module::game_module::LobbyGameModule;
use crate::resource_module::def::GuestId;
use crate::slime_1_module::def::Slime1SimulationConfig;
use crate::slime_1_module::game_module::generated::Slime1GameEntityManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type LobbyBasicGameModule = BasicGameModule<LobbyGameEntityManager, LobbySimulationConfig>;

pub struct LobbyModule {
    pub(crate) game_instance_manager:
        GameInstanceManager<LobbyGameEntityManager, LobbySimulationConfig, LobbyGameModule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LobbySimulationConfig {
    pub(crate) afk_time: u64,
}

pub struct LobbyModuleExitSlots {
    pub to_module_exit_1: &'static str,
    pub to_module_exit_2: &'static str,
}

pub struct LobbyModuleEnterSlots {
    pub main_enter: &'static str,
    pub module_1_enter: &'static str,
    pub module_2_enter: &'static str,
}
