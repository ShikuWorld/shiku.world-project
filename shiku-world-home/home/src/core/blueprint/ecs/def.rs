use rapier2d::control::KinematicCharacterController;
use std::collections::{BTreeMap, HashMap, HashSet};

use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::math::Vector;
use rapier2d::prelude::{ColliderHandle, Real};
use rhai::{CustomType, Dynamic, TypeBuilder};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use ts_rs::TS;

use crate::core::ApiShare;
use remove_entity::RemoveEntity;

use crate::core::blueprint::def::{Gid, LayerKind, ResourcePath};
use crate::core::blueprint::ecs::character_animation::CharacterAnimation;
use crate::core::blueprint::ecs::game_node_script::{GameNodeScript, ScopeCacheValue};
use crate::core::blueprint::scene::def::{
    Collider, GameNodeId, GameNodeKindClean, KinematicCharacterControllerProps, Node2DKindClean,
    NodeInstanceId, RenderKind, RenderKindClean, RigidBodyType, SceneId, Transform,
};

#[derive(
    TS, Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, CustomType,
)]
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
    pub processed_added_entities: Vec<Entity>,
    pub shared: ApiShare<ECSShared>,
}

#[derive(Debug)]
pub struct ECSShared {
    pub entities: EntityMaps,
    pub added_entities: Vec<(Entity, Option<ResourcePath>)>,
    pub set_scope_variables: HashMap<Entity, HashMap<String, ScopeCacheValue>>,
    pub removed_entities: Vec<Entity>,
    pub entity_counter: NodeInstanceId,
}

pub type DynamicMap = BTreeMap<SmartString<LazyCompact>, Dynamic>;

#[derive(Debug, RemoveEntity)]
pub struct EntityMaps {
    pub game_node_id: HashMap<Entity, GameNodeId>,
    pub game_node_name: HashMap<Entity, String>,
    pub game_node_children: HashMap<Entity, Vec<Entity>>,
    pub game_node_parent: HashMap<Entity, Entity>,
    pub game_node_kind: HashMap<Entity, GameNodeKindClean>,
    pub node_2d_kind: HashMap<Entity, Node2DKindClean>,
    pub node_2d_instance_path: HashMap<Entity, ResourcePath>,
    pub node_2d_entity_instance_parent: HashMap<Entity, Entity>,
    pub render_kind: HashMap<Entity, RenderKindClean>,
    pub render_offset: HashMap<Entity, (Real, Real)>,
    pub render_layer: HashMap<Entity, LayerKind>,
    pub render_gid: HashMap<Entity, Gid>,
    pub render_gid_tileset_path: HashMap<Entity, ResourcePath>,
    pub character_animation: HashMap<Entity, CharacterAnimation>,
    pub transforms: HashMap<Entity, Transform>,
    pub rigid_body_type: HashMap<Entity, RigidBodyType>,
    pub kinematic_character: HashMap<Entity, KinematicCharacter>,
    pub rigid_body_handle: HashMap<Entity, RigidBodyHandle>,
    pub collider: HashMap<Entity, Collider>,
    pub collider_handle: HashMap<Entity, ColliderHandle>,
    pub dirty: HashMap<Entity, bool>,
    pub view_dirty: HashMap<Entity, bool>,
}

#[derive(Debug, Clone)]
pub struct KinematicCharacter {
    pub controller: KinematicCharacterController,
    pub props: KinematicCharacterControllerProps,
    pub desired_translation: Vector<Real>,
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
    KinematicCharacterControllerProps(KinematicCharacterControllerProps),
    Collider(Collider),
    PositionRotation((Real, Real, Real)),
    RenderKind(RenderKind),
    AnimatedSpriteResource(ResourcePath),
    SpriteTilesetResource(ResourcePath),
    Gid(Gid),
}
