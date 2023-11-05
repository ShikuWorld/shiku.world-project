use std::collections::HashMap;
use std::time::Instant;

use snowflake::SnowflakeIdBucket;

use crate::core::blueprint::BlueprintService;
use crate::core::guest::{Admin, Guest, ModuleEnterSlot, ModuleExitSlot, ProviderUserId};
use crate::core::module::{ModuleIO, ModuleName, SystemCommunicationIO};
use crate::core::module_system::game_instance::{GameInstanceId, GameInstanceManager};
use crate::core::{blueprint, Snowflake};
use crate::login::login_manager::LoginManager;
use crate::persistence_module::PersistenceModule;
use crate::resource_module::def::ActorId;
use crate::webserver_module::def::WebServerModule;
use crate::{ResourceModule, WebsocketModule};

pub type ModuleMap = HashMap<ModuleName, GameInstanceManager>;
pub type ModuleInstanceToModuleMap = HashMap<GameInstanceId, ModuleName>;
pub type ModuleCommunicationMap = HashMap<ModuleName, ModuleIO>;

pub struct ConductorModule {
    pub(super) blueprint: blueprint::Conductor,
    pub(super) blueprint_service: BlueprintService,
    pub(super) websocket_module: WebsocketModule,
    pub(super) resource_module: ResourceModule,
    pub(super) persistence_module: PersistenceModule,
    pub(super) web_server_module: WebServerModule,
    pub(super) login_manager: LoginManager,
    pub(super) module_map: ModuleMap,
    pub(super) module_connection_map: HashMap<ModuleExitSlot, (ModuleName, ModuleEnterSlot)>,
    pub(super) module_communication_map: ModuleCommunicationMap,
    pub(super) guests: HashMap<Snowflake, Guest>,
    pub(super) admins: HashMap<Snowflake, Admin>,

    pub(super) ws_to_guest_map: HashMap<Snowflake, Snowflake>,
    pub(super) ws_to_admin_map: HashMap<Snowflake, Snowflake>,
    pub(super) provider_id_to_guest_map: HashMap<ProviderUserId, Snowflake>,
    pub(super) provider_id_to_admin_map: HashMap<ProviderUserId, Snowflake>,
    pub(super) session_id_to_guest_map: HashMap<String, Snowflake>,
    pub(super) session_id_to_admin_map: HashMap<String, Snowflake>,
    pub(super) guest_timeout_map: HashMap<ActorId, Instant>,
    pub(super) timeouts: Vec<ActorId>,

    pub(super) snowflake_gen: SnowflakeIdBucket,
    pub(super) system_to_guest_communication: SystemCommunicationIO,
    pub(super) system_to_admin_communication: SystemCommunicationIO,
}
