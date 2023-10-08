use crate::core::entity::render::CameraSettings;
use crate::core::game_instance::GameInstanceManager;
use crate::core::game_module_communication::GameModuleCommunicationCallbacks;
use crate::core::guest::Guest;
use crate::core::module::{
    EnterFailedState, EnterSuccessState, GameModule, LeaveFailedState, LeaveSuccessState,
    ModuleInputReceiver, ModuleName, ModuleOutputSender, ModuleState,
};
use crate::core::module_system::ModuleCommunicationCallbacks;
use crate::core::resource_json_generation::generate_resource_map_from_tiled_map;
use crate::lobby_module::def::{
    LobbyBasicGameModule, LobbyModule, LobbyModuleEnterSlots, LobbyModuleExitSlots,
    LobbySimulationConfig,
};
use crate::lobby_module::game_module::generated::LobbyGameEntityManager;
use crate::lobby_module::game_module::LobbyGameModule;
use crate::resource_module::def::ResourceFile;
use crate::SystemModule;
use log::debug;

impl SystemModule for LobbyModule {
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

impl GameModule for LobbyModule {
    fn get_base_resource_file(&self) -> ResourceFile {
        generate_resource_map_from_tiled_map(
            self.module_name().as_str(),
            "lobby_module/resources/private/lobby.tmx",
        )
    }

    fn get_resource_json(&self) -> String {
        String::from_utf8_lossy(include_bytes!("lobby_module.resources.json")).to_string()
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

impl LobbyModule {
    pub fn module_name() -> String {
        "LobbyModule".to_string()
    }

    pub const EXIT_SLOTS: LobbyModuleExitSlots = LobbyModuleExitSlots {
        to_module_exit_1: "to_module_exit_1",
        to_module_exit_2: "to_module_exit_2",
    };

    pub const ENTER_SLOTS: LobbyModuleEnterSlots = LobbyModuleEnterSlots {
        main_enter: "lobby_enter",
        module_1_enter: "module_1_enter",
        module_2_enter: "module_2_enter",
    };

    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> LobbyModule {
        LobbyModule {
            game_instance_manager: GameInstanceManager::new(
                input_receiver,
                output_sender,
                0,
                100,
                false,
                LobbySimulationConfig { afk_time: 30 },
                "lobby_module/resources/private/lobby.tmx".to_string(),
                Self::module_name(),
                Self::create_game_entity_manager,
                Self::create_game_module,
            ),
        }
    }

    pub fn create_game_entity_manager() -> LobbyGameEntityManager {
        LobbyGameEntityManager::new()
    }

    pub fn create_game_module(basic_game_module: &mut LobbyBasicGameModule) -> LobbyGameModule {
        basic_game_module.base_camera_settings = Some(CameraSettings {
            zoom: Some(0.5),
            bounds: None,
        });
        LobbyGameModule::new(basic_game_module)
    }
}
