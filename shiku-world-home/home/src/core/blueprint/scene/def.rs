use crate::core::blueprint::def::{Gid, LayerKind, ResourcePath};
use crate::core::blueprint::ecs::def::Entity;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub type SceneId = String;
pub type GameNodeId = String;
pub type NodeInstanceId = u32;
#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Scene {
    pub id: SceneId,
    pub name: String,
    pub resource_path: ResourcePath,
    pub root_node: GameNodeKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum GameNodeKind {
    Instance(GameNode<ResourcePath>),
    Node2D(GameNode<Node2D>),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum GameNodeKindClean {
    Instance,
    Node2D,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct GameNode<T> {
    pub id: GameNodeId,
    pub name: String,
    pub entity_id: Option<Entity>,
    pub data: T,
    pub script: Option<String>,
    pub children: Vec<GameNodeKind>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Node2D {
    pub transform: Transform,
    pub kind: Node2DKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Node2DDud(pub usize);

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum Node2DKind {
    Node2D(Node2DDud),
    RigidBody(RigidBody),
    Collider(Collider),
    Render(Render),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum Node2DKindClean {
    Node2D,
    RigidBody,
    Collider,
    Render,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Collider {
    pub kind: ColliderKind,
    pub shape: ColliderShape,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ColliderShape {
    Ball(f32),
    CapsuleX(f32, f32),
    CapsuleY(f32, f32),
    Cuboid(f32, f32),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ColliderKind {
    Solid,
    Sensor,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Transform {
    pub position: (Real, Real),
    pub scale: (Real, Real),
    pub velocity: (Real, Real),
    pub rotation: Real,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct RigidBody {
    pub velocity: (Real, Real),
    pub body: RigidBodyType,
}
#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum RigidBodyType {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum CollisionShape {}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Render {
    pub offset: (Real, Real),
    pub layer: LayerKind,
    pub kind: RenderKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum RenderKind {
    AnimatedSprite(Gid),
    Sprite(Gid),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum RenderKindClean {
    AnimatedSprite,
    Sprite,
}