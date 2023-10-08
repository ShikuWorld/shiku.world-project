use crate::core::game_module_communication::GameModuleCommunication;

use crate::slime_1_module::game_module::Slime1GameModule;

use crate::core::basic_game_module::BasicGameModule;
use crate::core::game_instance::GameInstanceManager;
use crate::slime_1_module::game_module::generated::Slime1GameEntityManager;
use rapier2d::prelude::Real;
use serde::{Deserialize, Serialize};

pub type Slime1BasicGameModule = BasicGameModule<Slime1GameEntityManager, Slime1SimulationConfig>;

pub struct Slime1Module {
    pub(crate) game_instance_manager:
        GameInstanceManager<Slime1GameEntityManager, Slime1SimulationConfig, Slime1GameModule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Slime1SimulationConfig {
    pub(crate) gravity: (Real, Real),
    pub(crate) guest_move_force: Real,
    pub(crate) guest_jump_move_force: Real,
    pub(crate) guest_jump_force: Real,
    pub(crate) guest_in_air_nudging_force: Real,
    pub(crate) guest_linear_dampening: Real,
    pub(crate) guest_bounciness: Real,
    pub(crate) light_power_multiplier: Real,
    pub(crate) afk_time: u64,
}
