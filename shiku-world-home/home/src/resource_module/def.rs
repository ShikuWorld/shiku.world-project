use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::core::blueprint::def::ResourcePath;
use crate::core::guest::ActorId;
use flume::Receiver;
use notify::event::ModifyKind;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdBucket;
use ts_rs::TS;

use crate::core::module::{GuestEvent, ModuleInstanceEvent, ModuleName};
use crate::core::Snowflake;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum LoadResourceKind {
    Image,
    Unknown,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Resource {
    pub(super) kind: LoadResourceKind,
    pub(super) path: ResourcePath,
    pub(super) cache_hash: Snowflake,
}
#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ResourceBundle {
    pub name: String,
    pub assets: Vec<Resource>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ResourceEvent {
    LoadResource(ResourceBundle),
    UnLoadResources,
}

#[derive(Debug, Deserialize)]
pub struct PicUpdateEvent {
    pub path: String,
    pub kind: ModifyKind,
}

pub struct ResourceModule {
    pub(super) active_resources: HashMap<ActorId, HashMap<ModuleName, bool>>,
    pub(super) resources: HashMap<ModuleName, HashMap<ResourcePath, Resource>>,
    pub(super) resource_load_events: Vec<GuestEvent<ModuleInstanceEvent<ResourceEvent>>>,
    pub(super) resource_hash_gen: SnowflakeIdBucket,
    pub(super) pic_changed_events_hash: HashSet<String>,
    pub(super) pic_update_receiver: Receiver<PicUpdateEvent>,
    pub(super) last_insert: Instant,
}
