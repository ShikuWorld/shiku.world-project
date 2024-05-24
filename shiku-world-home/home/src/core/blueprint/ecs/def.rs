use std::collections::{BTreeMap, HashMap, HashSet};

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::prelude::{ColliderHandle, Real};
use rhai::{Dynamic, ParseError, Scope, AST};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ts_rs::TS;

use crate::core::ApiShare;
use remove_entity::RemoveEntity;

use crate::core::blueprint::def::{BlueprintError, Gid, LayerKind, ResourcePath};
use crate::core::blueprint::scene::def::{
    Collider, GameNodeId, GameNodeKindClean, Node2DKindClean, NodeInstanceId, RenderKindClean,
    RigidBodyType, SceneId, Transform,
};

#[derive(TS, Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[ts(export, export_to = "blueprints/")]
pub struct Entity(pub NodeInstanceId);

#[derive(Debug)]
pub struct ECS {
    pub scene_root: Entity,
    pub scene_name: String,
    pub scene_resource_path: ResourcePath,
    pub scene_id: SceneId,
    pub entities: HashSet<Entity>,
    pub entity_scripts: HashMap<Entity, GameNodeScript>,
    pub shared: ApiShare<ECSShared>,
}

#[derive(Debug)]
pub struct ECSShared {
    pub entities: EntityMaps,
    pub added_entities: Vec<(Entity, Option<ResourcePath>)>,
    pub removed_entities: Vec<Entity>,
    pub entity_counter: NodeInstanceId,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ScopeCacheValue {
    String(String),
    Number(f64),
    Integer(i64),
    Map(HashMap<String, ScopeCacheValue>),
}

impl ScopeCacheValue {
    pub(crate) fn equals_dynamic_value(
        scope_cache_value: &ScopeCacheValue,
        dynamic_value: &Dynamic,
    ) -> bool {
        match scope_cache_value {
            ScopeCacheValue::String(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<String>() {
                    *value == *dynamic_value
                } else {
                    false
                }
            }
            ScopeCacheValue::Number(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<f64>() {
                    (value - *dynamic_value).abs() < 0.0001_f64
                } else {
                    false
                }
            }
            ScopeCacheValue::Integer(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<i64>() {
                    *value == *dynamic_value
                } else {
                    false
                }
            }
            ScopeCacheValue::Map(scope_cache_map) => {
                if let Some(dynamic_map) = dynamic_value.read_lock::<DynamicMap>() {
                    scope_cache_map.iter().all(|(key, cache_val)| {
                        let smart_string: SmartString<LazyCompact> = key.into();
                        match dynamic_map.get(&smart_string) {
                            Some(dyn_val) => {
                                ScopeCacheValue::equals_dynamic_value(cache_val, dyn_val)
                            }
                            None => false,
                        }
                    })
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GameNodeScript {
    pub path: ResourcePath,
    pub ast: AST,
    pub entity: Entity,
    pub scope_cache: HashMap<String, ScopeCacheValue>,
    pub scope: Scope<'static>,
    pub(crate) game_node_script_functions: GameNodeScriptFunctions,
}

#[derive(Debug)]
pub struct GameNodeScriptFunctions {
    pub init: bool,
    pub update: bool,
    pub actor_joined: bool,
    pub actor_left: bool,
}

pub type DynamicMap = BTreeMap<SmartString<LazyCompact>, Dynamic>;

#[derive(Debug)]
pub enum GameNodeScriptError {
    BlueprintError(BlueprintError),
    CompileError(ParseError),
}

#[derive(Debug, RemoveEntity)]
pub struct EntityMaps {
    pub game_node_id: HashMap<Entity, GameNodeId>,
    pub game_node_name: HashMap<Entity, String>,
    pub game_node_children: HashMap<Entity, Vec<Entity>>,
    pub game_node_kind: HashMap<Entity, GameNodeKindClean>,
    pub node_2d_kind: HashMap<Entity, Node2DKindClean>,
    pub node_2d_instance_path: HashMap<Entity, ResourcePath>,
    pub node_2d_entity_instance_parent: HashMap<Entity, Entity>,
    pub render_kind: HashMap<Entity, RenderKindClean>,
    pub render_offset: HashMap<Entity, (Real, Real)>,
    pub render_layer: HashMap<Entity, LayerKind>,
    pub render_gid: HashMap<Entity, Gid>,
    pub transforms: HashMap<Entity, Transform>,
    pub rigid_body_velocity: HashMap<Entity, (Real, Real)>,
    pub rigid_body_type: HashMap<Entity, RigidBodyType>,
    pub rigid_body_handle: HashMap<Entity, RigidBodyHandle>,
    pub collider: HashMap<Entity, Collider>,
    pub collider_handle: HashMap<Entity, ColliderHandle>,
    pub dirty: HashMap<Entity, bool>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct EntityUpdate {
    pub id: Entity,
    pub kind: EntityUpdateKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum EntityUpdateKind {
    Transform(Transform),
    Name(String),
    InstancePath(ResourcePath),
    ScriptPath(Option<ResourcePath>),
    UpdateScriptScope(String, ScopeCacheValue),
    SetScriptScope(HashMap<String, ScopeCacheValue>),
    RigidBodyType(RigidBodyType),
    PositionRotation((Real, Real, Real)),
    Gid(Gid),
}
