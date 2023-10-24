use std::collections::{HashMap, HashSet};

use apecs::World;
use tokio::time::Instant;

use crate::core::blueprint;
use crate::core::module::{GuestInput, ModuleInputReceiver, ModuleOutputSender};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::resource_module::def::GuestId;

pub struct GuestCommunication {
    pub resources_loaded: bool,
    pub connected: bool,
}

pub struct ModuleCommunication {
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
}

impl ModuleCommunication {
    pub fn new(
        input_receiver: ModuleInputReceiver,
        output_sender: ModuleOutputSender,
    ) -> ModuleCommunication {
        ModuleCommunication {
            input_receiver,
            output_sender,
        }
    }
}

pub type GuestMap = HashMap<GuestId, ModuleGuest>;
pub type AdminSet = HashSet<GuestId>;

pub struct DynamicGameModule {
    pub world: World,
    pub blueprint: blueprint::Module,
    pub guests: GuestMap,
    pub admins: AdminSet,
    pub module_communication: ModuleCommunication,
    pub instance_id: GameInstanceId,
}

pub struct ModuleGuest {
    pub(crate) id: GuestId,
    pub(crate) guest_input: GuestInput,
    pub(crate) guest_com: GuestCommunication,
    pub(crate) last_input_time: Instant,
}

pub struct ModuleService {
    pub(super) available_modules: HashMap<String, DynamicGameModule>,
}
