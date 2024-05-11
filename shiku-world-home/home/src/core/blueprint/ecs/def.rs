use std::collections::HashMap;

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::prelude::{ColliderHandle, Real};
use remove_entity::RemoveEntity;
use rhai::Scope;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::blueprint::def::{Gid, LayerKind, ResourcePath};
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
    pub entities: EntityMaps,
    pub entity_counter: NodeInstanceId,
}

#[derive(Debug, RemoveEntity)]
pub struct EntityMaps {
    pub game_node_script: HashMap<Entity, (ResourcePath, Scope<'static>)>,
    pub game_node_id: HashMap<Entity, GameNodeId>,
    pub game_node_name: HashMap<Entity, String>,
    pub game_node_children: HashMap<Entity, Vec<Entity>>,
    pub game_node_kind: HashMap<Entity, GameNodeKindClean>,
    pub node_2d_kind: HashMap<Entity, Node2DKindClean>,
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
    RigidBodyType(RigidBodyType),
    PositionRotation((Real, Real, Real)),
    Gid(Gid),
}
