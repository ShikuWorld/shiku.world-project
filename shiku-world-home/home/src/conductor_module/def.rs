use std::collections::HashMap;
use std::time::Instant;

use snowflake::SnowflakeIdBucket;

use crate::core::blueprint::BlueprintService;
use crate::core::guest::{Admin, Guest, ModuleEnterSlot, ModuleExitSlot, ProviderUserId};
use crate::core::module::{ModuleIO, ModuleName};
use crate::core::module_system::DynamicGameModule;
use crate::core::{blueprint, Snowflake};
use crate::persistence_module::PersistenceModule;
use crate::resource_module::def::GuestId;
use crate::webserver_module::def::WebServerModule;
use crate::{ResourceModule, WebsocketModule};

pub struct ConductorModule {
    pub(super) blueprint: blueprint::Conductor,
    pub(super) blueprint_service: BlueprintService,
    pub(super) websocket_module: WebsocketModule,
    pub(super) resource_module: ResourceModule,
    pub(super) persistence_module: PersistenceModule,
    #[allow(dead_code)]
    pub(super) web_server_module: WebServerModule,
    pub(super) module_map: HashMap<ModuleName, DynamicGameModule>,
    pub(super) guests: HashMap<Snowflake, Guest>,
    pub(super) admins: HashMap<Snowflake, Admin>,

    pub(super) ws_to_guest_map: HashMap<Snowflake, Snowflake>,
    pub(super) provider_id_to_guest_map: HashMap<ProviderUserId, Snowflake>,
    pub(super) provider_id_to_admin_map: HashMap<ProviderUserId, Snowflake>,
    pub(super) session_id_to_guest_map: HashMap<String, Snowflake>,
    pub(super) session_id_to_admin_map: HashMap<String, Snowflake>,
    pub(super) guest_timeout_map: HashMap<GuestId, Instant>,
    pub(super) module_connection_map: HashMap<ModuleExitSlot, (ModuleName, ModuleEnterSlot)>,
    pub(super) timeouts: Vec<GuestId>,

    pub(super) snowflake_gen: SnowflakeIdBucket,
    pub(super) module_communication_map: HashMap<ModuleName, ModuleIO>,
}
