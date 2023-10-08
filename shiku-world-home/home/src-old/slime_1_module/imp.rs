use log::{debug, error};
use rapier2d::prelude::Vector;

use std::fs::read_to_string;

use crate::core::game_module_communication::{
    GameModuleCommunication, GameModuleCommunicationCallbacks,
};
use crate::core::guest::Guest;
use crate::core::module::{
    EnterFailedState, EnterSuccessState, GameModule, LeaveFailedState, LeaveSuccessState,
    ModuleInputReceiver, ModuleName, ModuleOutputSender, ModuleState, SystemModule,
};
use crate::core::{get_out_dir, safe_unwrap_ref};
use crate::slime_1_module::def::{Slime1BasicGameModule, Slime1Module, Slime1SimulationConfig};
use crate::slime_1_module::game_module::Slime1GameModule;

use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity::render::CameraSettings;
use crate::core::game_instance::GameInstanceManager;
use crate::core::resource_json_generation::generate_resource_map_from_tiled_map;
use crate::resource_module::def::ResourceFile;
use crate::resource_module::errors::ResourceParseError;
use crate::slime_1_module::game_module::generated::Slime1GameEntityManager;

impl SystemModule for Slime1Module {
    fn module_name(&self) -> ModuleName {
        Self::module_name()
    }

    fn status(&self) -> &ModuleState {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn shutdown(&mut self) {
        todo!()
    }
}

impl GameModule for Slime1Module {
    fn get_base_resource_file(&self) -> ResourceFile {
        generate_resource_map_from_tiled_map(
            self.module_name().as_str(),
            "slime_1_module/resources/private/map.tmx",
        )
    }
    fn get_resource_json(&self) -> String {
        String::from_utf8_lossy(include_bytes!("slime_1.resources.json")).to_string()
    }

    fn update(&mut self) {
        self.game_instance_manager.update();
    }

    fn try_enter(
        &mut self,
        guest: &Guest,
        entry_point_id: &String,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.game_instance_manager.try_enter(guest, entry_point_id)
    }

    fn try_leave(&mut self, guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        self.game_instance_manager.try_leave(guest)
    }
}

pub struct Slime1ModuleExitSlots {
    pub cave_forest_exit: &'static str,
    pub to_lobby_exit: &'static str,
}

pub struct Slime1ModuleEnterSlots {
    pub cave_entry: &'static str,
    pub cave_forest_entry: &'static str,
}

impl Slime1Module {
    pub const EXIT_SLOTS: Slime1ModuleExitSlots = Slime1ModuleExitSlots {
        cave_forest_exit: "cave_forest_exit",
        to_lobby_exit: "to_lobby_exit",
    };

    pub const ENTER_SLOTS: Slime1ModuleEnterSlots = Slime1ModuleEnterSlots {
        cave_entry: "cave_entry",
        cave_forest_entry: "cave_forest_entry",
    };

    pub fn module_name() -> ModuleName {
        "Slime1Module".to_string()
    }

    pub fn load_simulation_config() -> Option<Slime1SimulationConfig> {
        let path_to_config = get_out_dir()
            .join("slime_1_module")
            .join("resources")
            .join("private")
            .join("simulation_config.json");
        if let Ok(simulation_config_as_string) = read_to_string(path_to_config) {
            if let Ok(config) = serde_json::from_str(simulation_config_as_string.as_str()) {
                debug!("Config successfully loaded!");
                return Some(config);
            }
        }
        error!("Could not load slime simulation config!");
        None
    }

    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> Slime1Module {
        let simulation_config = Self::load_simulation_config().unwrap_or(Slime1SimulationConfig {
            gravity: (0.0, 100.0),
            guest_move_force: 0.5,
            guest_jump_force: -5000.0,
            guest_in_air_nudging_force: 5000.0,
            guest_linear_dampening: 0.0,
            guest_jump_move_force: 0.5,
            guest_bounciness: 0.0,
            afk_time: 0,
            light_power_multiplier: 3.0,
        });

        Slime1Module {
            game_instance_manager: GameInstanceManager::new(
                input_receiver,
                output_sender,
                0,
                100,
                false,
                simulation_config,
                "slime_1_module/resources/private/map.tmx".to_string(),
                Self::module_name(),
                Self::create_slime_1_entity_manager,
                Self::create_slime_1_game_module,
            ),
        }
    }

    fn create_slime_1_entity_manager() -> Slime1GameEntityManager {
        Slime1GameEntityManager::new()
    }

    fn create_slime_1_game_module(
        basic_game_module: &mut Slime1BasicGameModule,
    ) -> Slime1GameModule {
        let gravity = basic_game_module.simulation_config.gravity;
        basic_game_module.set_gravity(Vector::new(gravity.0, gravity.1));
        basic_game_module.base_camera_settings = Some(CameraSettings {
            zoom: Some(0.5),
            bounds: None,
        });
        Slime1GameModule::new(basic_game_module)
    }
}
