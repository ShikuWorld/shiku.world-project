use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::math::Vector;
use rapier2d::prelude::{ColliderHandle, Real};
use rhai::{CustomType, Dynamic, TypeBuilder};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;
use ts_rs::TS;

use crate::core::basic_kinematic_character_controller::{
    BasicKinematicCharacterController, CharacterCollision,
};
use crate::core::blueprint::def::{Gid, LayerKind, ResourcePath};
use crate::core::blueprint::ecs::character_animation::CharacterAnimation;
use crate::core::blueprint::ecs::game_node_script::{GameNodeScript, ScopeCacheValue};
use crate::core::blueprint::scene::def::{
    Collider, GameNodeId, GameNodeKindClean, KinematicCharacterControllerProps, Node2DKindClean,
    NodeInstanceId, RenderKind, RenderKindClean, RigidBodyType, SceneId, TextRender, Transform,
};
use crate::core::timer::Timer;
use crate::core::tween::Tween;
use crate::core::ApiShare;
use remove_entity::RemoveEntity;

#[derive(
    TS,
    Serialize,
    Deserialize,
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    CustomType,
)]
#[ts(export, export_to = "blueprints/")]
pub struct Entity(pub NodeInstanceId);

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

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

pub type TweenId = u64;
pub type TimerId = u64;

#[derive(Debug)]
pub struct ECSShared {
    pub entities: EntityMaps,
    pub added_entities: Vec<(Entity, Option<ResourcePath>)>,
    pub set_scope_variables: HashMap<Entity, HashMap<String, ScopeCacheValue>>,
    pub removed_entities: Vec<Entity>,
    pub entity_counter: NodeInstanceId,
    pub tween_map: HashMap<TweenId, Tween>,
    pub timer_map: HashMap<TimerId, Timer>,
    pub timer_counter: TimerId,
    pub tween_counter: TweenId,
    pub character_collisions_tmp: Vec<CharacterCollision>,
    pub collider_to_entity_map: HashMap<ColliderHandle, Entity>,
    pub kinematic_collision_map: HashMap<Entity, (CharacterCollision, ColliderHandle, bool)>,
}

pub type DynamicMap = BTreeMap<SmartString<LazyCompact>, Dynamic>;

#[derive(Debug, RemoveEntity)]
pub struct EntityMaps {
    pub game_node_id: HashMap<Entity, GameNodeId>,
    pub game_node_name: HashMap<Entity, String>,
    pub game_node_children: HashMap<Entity, Vec<Entity>>,
    pub game_node_parent: HashMap<Entity, Entity>,
    pub game_node_kind: HashMap<Entity, GameNodeKindClean>,
    pub game_node_tags: HashMap<Entity, Vec<String>>,
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
    pub text_render: HashMap<Entity, TextRender>,
    pub dirty: HashMap<Entity, bool>,
    pub view_dirty: HashMap<Entity, bool>,
}

#[derive(Debug, Clone)]
pub struct KinematicCharacter {
    pub controller: BasicKinematicCharacterController,
    pub props: KinematicCharacterControllerProps,
    pub desired_translation: Vector<Real>,
    pub grounded: bool,
    pub is_sliding_down_slope: bool,
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
    Tags(Vec<String>),
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
    TextRender(TextRender),
    Gid(Gid),
}
