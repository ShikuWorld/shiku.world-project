use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::core::blueprint::def::{GidMap, ModuleId, ResourcePath, Tileset};
use crate::core::guest::ActorId;
use flume::Receiver;
use notify::event::ModifyKind;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdBucket;
use tokio::task::JoinHandle;
use ts_rs::TS;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum LoadResourceKind {
    Image,
    Unknown,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct LoadResource {
    pub(super) kind: LoadResourceKind,
    pub(super) path: ResourcePath,
    pub(super) cache_hash: String,
}

impl LoadResource {
    pub fn image(path: ResourcePath) -> LoadResource {
        LoadResource {
            cache_hash: String::default(),
            kind: LoadResourceKind::Image,
            path,
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct ResourceBundle {
    pub name: String,
    pub assets: Vec<LoadResource>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub enum ResourceEvent {
    LoadTilesets(Vec<Tileset>),
    UpdateGidMap(GidMap),
    LoadResource(ResourceBundle),
    UnLoadResources,
}

#[derive(Debug, Deserialize)]
pub struct PicUpdateEvent {
    pub path: String,
    pub kind: ModifyKind,
}

pub struct ResourceModuleBookKeeping {
    pub(super) active_resources: HashMap<ActorId, HashMap<ModuleId, bool>>,
    pub(super) path_to_module_map: HashMap<ResourcePath, ModuleId>,
    pub(super) module_actor_set: HashMap<ModuleId, HashSet<ActorId>>,
    pub(super) resources: HashMap<ModuleId, HashMap<ResourcePath, LoadResource>>,
    pub(super) resource_hash_gen: SnowflakeIdBucket,
}

pub struct PicUpdateWSConnection {
    pub(super) receiver: Receiver<PicUpdateEvent>,
    pub(super) join_handle: JoinHandle<()>,
}

pub struct ResourceModulePicUpdates {
    pub(super) pic_changed_events_hash: HashSet<String>,
    pub(super) pic_update_ws_connection: PicUpdateWSConnection,
    pub(super) last_insert: Instant,
}

pub struct ResourceModule {
    pub(super) book_keeping: ResourceModuleBookKeeping,
    pub(super) pic_updates: ResourceModulePicUpdates,
    pub(super) resource_events: Vec<(ActorId, ModuleId, ResourceEvent)>,
}
