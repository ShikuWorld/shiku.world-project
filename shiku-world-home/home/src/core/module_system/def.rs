use std::collections::{HashMap, HashSet};

use tokio::time::Instant;

use crate::core::blueprint::def::ModuleId;
use crate::core::guest::ActorId;
use crate::core::LazyHashmapSet;
use crate::core::module::{GuestInput, ModuleInputReceiver, ModuleOutputSender};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::module_system::world::{World, WorldId};

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

pub type GuestMap = HashMap<ActorId, ModuleGuest>;
pub type AdminMap = HashMap<ActorId, ModuleAdmin>;
pub type AdminSet = HashSet<ActorId>;

pub struct DynamicGameModule {
    pub world_map: HashMap<WorldId, World>,
    pub guests: GuestMap,
    pub admins: AdminMap,
    pub world_to_admin: LazyHashmapSet<WorldId, ActorId>,
    pub world_to_guest: LazyHashmapSet<WorldId, ActorId>,
    pub admin_to_world: LazyHashmapSet<ActorId, WorldId>,
    pub guest_to_world: HashMap<ActorId, WorldId>,
    pub module_communication: ModuleCommunication,
    pub instance_id: GameInstanceId,
    pub module_id: ModuleId,
}

pub struct ModuleGuest {
    pub(crate) id: ActorId,
    pub(crate) world_id: Option<WorldId>,
    pub(crate) guest_input: GuestInput,
    pub(crate) guest_com: GuestCommunication,
    pub(crate) last_input_time: Instant,
}

pub struct ModuleAdmin {
    pub(crate) id: ActorId,
    pub(crate) resources_loaded: bool,
    pub(crate) connected: bool,
}

pub struct ModuleService {
    pub(super) available_modules: HashMap<String, DynamicGameModule>,
}
