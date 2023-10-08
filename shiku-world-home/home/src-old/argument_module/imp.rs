use crate::argument_module::def::{
    ArgumentBasicGameModule, ArgumentModule, ArgumentModuleEnterSlots, ArgumentModuleExitSlots,
    ArgumentSimulationConfig,
};
use crate::argument_module::game_module::generated::ArgumentGameEntityManager;
use crate::argument_module::game_module::ArgumentGameModule;
use crate::core::basic_game_module::BasicGameModule;
use crate::core::entity::render::CameraSettings;
use crate::core::game_instance::GameInstanceManager;
use crate::core::game_module_communication::{
    GameModuleCommunication, GameModuleCommunicationCallbacks,
};
use crate::core::guest::{Guest, ModuleEnterSlot};
use crate::core::module::{
    EnterFailedState, EnterSuccessState, GameModule, LeaveFailedState, LeaveSuccessState,
    ModuleInputReceiver, ModuleName, ModuleOutputSender, ModuleState, SystemModule,
};
use crate::core::module_system::{ModuleCommunication, ModuleCommunicationCallbacks};
use crate::core::resource_json_generation::generate_resource_map_from_tiled_map;
use crate::core::resource_watcher::ResourceWatcher;
use crate::core::{get_out_dir, safe_unwrap_ref};
use crate::resource_module::def::ResourceFile;
use crate::resource_module::errors::ResourceParseError;
use log::{debug, error};
use notify_debouncer_mini::DebouncedEvent;
use rapier2d::prelude::Vector;
use std::fs::read_to_string;

impl SystemModule for ArgumentModule {
    fn module_name(&self) -> ModuleName {
        Self::module_name()
    }

    fn status(&self) -> &ModuleState {
        &ModuleState::Starting
    }

    fn start(&mut self) {
        debug!("start");
    }

    fn shutdown(&mut self) {
        debug!("shutdown");
    }
}

impl GameModule for ArgumentModule {
    fn get_base_resource_file(&self) -> ResourceFile {
        generate_resource_map_from_tiled_map(
            self.module_name().as_str(),
            "argument_module/resources/private/argument.tmx",
        )
    }

    fn get_resource_json(&self) -> String {
        String::from_utf8_lossy(include_bytes!("argument_module.resources.json")).to_string()
    }

    fn update(&mut self) {
        self.process_resource_changes();
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

impl ArgumentModule {
    pub fn module_name() -> String {
        "ArgumentModule".to_string()
    }

    pub const EXIT_SLOTS: ArgumentModuleExitSlots = ArgumentModuleExitSlots {
        exit: "argument_exit",
    };

    pub const ENTER_SLOTS: ArgumentModuleEnterSlots = ArgumentModuleEnterSlots {
        enter: "argument_enter",
    };

    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> ArgumentModule {
        let simulation_config =
            Self::load_simulation_config().unwrap_or_else(|| ArgumentSimulationConfig {
                afk_time: 30,
                stick_offset_x: 50.0,
                stick_offset_y: 50.0,
                throw_force_x: 0.05,
                throw_force_y: -1.00,
                stick_gravity: 0.09,
            });
        ArgumentModule {
            resource_watcher: ResourceWatcher::new("argument_module/resources/private"),
            game_instance_manager: GameInstanceManager::new(
                input_receiver,
                output_sender,
                0,
                2,
                true,
                simulation_config,
                "argument_module/resources/private/argument.tmx".to_string(),
                Self::module_name(),
                Self::create_game_entity_manager,
                Self::create_game_module,
            ),
        }
    }

    pub fn load_simulation_config() -> Option<ArgumentSimulationConfig> {
        let path_to_config = get_out_dir()
            .join("argument_module")
            .join("resources")
            .join("private")
            .join("simulation_config.json");

        match read_to_string(path_to_config) {
            Ok(simulation_config_as_string) => {
                if let Ok(config) = serde_json::from_str(simulation_config_as_string.as_str()) {
                    debug!("Config successfully loaded!");
                    return Some(config);
                }
            }
            Err(err) => error!("{:?}", err),
        }
        error!("Could not load slime simulation config!");
        None
    }

    pub fn create_game_entity_manager() -> ArgumentGameEntityManager {
        ArgumentGameEntityManager::new()
    }

    pub fn create_game_module(
        basic_game_module: &mut ArgumentBasicGameModule,
    ) -> ArgumentGameModule {
        basic_game_module.base_camera_settings = Some(CameraSettings {
            zoom: Some(1.0),
            bounds: Some(((-16.0, -6.0), (9.0, 11.0))),
        });
        ArgumentGameModule::new()
    }

    fn process_resource_changes(&mut self) {
        for debounced_event_result in self.resource_watcher.receiver.try_iter() {
            if let Ok(debounced_events) = debounced_event_result {
                for debounced_event in debounced_events {
                    if let Some(file_name) = debounced_event.path.file_name() {
                        if file_name == "simulation_config.json" {
                            if let Some(config) = Self::load_simulation_config() {
                                for game_instance in
                                    self.game_instance_manager.game_instances.values_mut()
                                {
                                    game_instance.basic_game_module.simulation_config =
                                        config.clone();
                                }
                                debug!("Set configs");
                            } else {
                                debug!("Reloading config failed, staying at the old config.");
                            }
                        }
                    }
                }
            }
        }
    }
}
