use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::blueprint::def::{Gid, LayerKind, ResourcePath};
use crate::core::blueprint::ecs::def::{DynamicRigidBodyPropsUpdate, Entity, ProgressBarUpdate};

pub type SceneId = String;
pub type ScriptId = String;
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
pub struct Script {
    pub id: ScriptId,
    pub name: String,
    pub resource_path: ResourcePath,
    pub content: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum GameNodeKind {
    Node2D(GameNode<Node2D>),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum GameNodeKindClean {
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
    pub tags: Vec<String>,
    pub instance_resource_path: Option<ResourcePath>,
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
    Instance(ResourcePath),
    Node2D(Node2DDud),
    RigidBody(RigidBody),
    Collider(Collider),
    Render(Render),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum Node2DKindClean {
    Instance,
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
    pub density: Real,
    pub restitution: Real,
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

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            scale: (1.0, 1.0),
            velocity: (0.0, 0.0),
            rotation: 0.0,
        }
    }
}

impl Transform {
    pub fn from_position(position: (Real, Real)) -> Self {
        Self {
            position,
            scale: (1.0, 1.0),
            velocity: (0.0, 0.0),
            rotation: 0.0,
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct AutoStepProps {
    pub max_height: f32,
    pub min_width: f32,
    pub include_dynamic_bodies: bool,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct KinematicCharacterControllerProps {
    pub offset: Real,
    pub up: (Real, Real),
    pub slide: bool,
    pub autostep: Option<AutoStepProps>,
    pub max_slope_climb_angle: Real,
    pub min_slope_slide_angle: Real,
    pub snap_to_ground: Option<Real>,
    pub normal_nudge_factor: Real,
}

impl KinematicCharacterControllerProps {
    pub fn new() -> Self {
        Self {
            offset: 0.001,
            up: (0.0, -1.0),
            slide: true,
            autostep: None,
            max_slope_climb_angle: 45.0,
            min_slope_slide_angle: 30.0,
            snap_to_ground: None,
            normal_nudge_factor: 0.001,
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct DynamicRigidBodyProps {
    pub gravity_scale: Real,
    pub can_sleep: bool,
    pub ccd_enabled: bool,
    pub linear_dampening: Real,
    pub angular_dampening: Real,
    pub rotation_locked: bool,
}

impl DynamicRigidBodyProps {
    pub fn new() -> Self {
        Self {
            gravity_scale: 1.0,
            can_sleep: true,
            ccd_enabled: false,
            linear_dampening: 0.0,
            angular_dampening: 0.0,
            rotation_locked: false,
        }
    }

    pub fn update(&mut self, update: DynamicRigidBodyPropsUpdate) {
        if let Some(gravity_scale) = update.gravity_scale {
            self.gravity_scale = gravity_scale;
        }
        if let Some(can_sleep) = update.can_sleep {
            self.can_sleep = can_sleep;
        }
        if let Some(ccd_enabled) = update.ccd_enabled {
            self.ccd_enabled = ccd_enabled;
        }
        if let Some(linear_dampening) = update.linear_dampening {
            self.linear_dampening = linear_dampening;
        }
        if let Some(angular_dampening) = update.angular_dampening {
            self.angular_dampening = angular_dampening;
        }
        if let Some(rotation_locked) = update.rotation_locked {
            self.rotation_locked = rotation_locked;
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct RigidBody {
    pub kinematic_character_controller_props: Option<KinematicCharacterControllerProps>,
    pub dynamic_rigid_body_props: Option<DynamicRigidBodyProps>,
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
pub enum CollisionShape {
    Rectangle(Real, Real, Real, Real), // start_x, start_y, width, height
    Circle(Real, Real, Real),          // center_x, center_y, radius
    Polygon(Vec<(Real, Real)>),        // x,y of a closed polygon
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum FadeinEffect {
    None,
    Fade,
    JumpForth,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum FadeoutEffect {
    None,
    Fade,
    JumpBack,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Render {
    pub offset: (Real, Real),
    pub layer: LayerKind,
    pub fadein_effect: (FadeinEffect, u32),
    pub fadeout_effect: (FadeoutEffect, u32),
    pub kind: RenderKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum RenderKind {
    AnimatedSprite(ResourcePath, Gid),
    Sprite(ResourcePath, Gid),
    Text(TextRender),
    ProgressBar(ProgressBar),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct ProgressBar {
    pub progress: Real,
    pub tileset: ResourcePath,
    pub background: Gid,
    pub fill: Gid,
    pub fill_paddings: (Real, Real, Real, Real),
    pub width: Real,
    pub height: Real,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            tileset: ResourcePath::new(),
            background: 0,
            fill: 0,
            width: 50.0,
            height: 50.0,
            fill_paddings: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn update(&mut self, update: ProgressBarUpdate) {
        if let Some(tileset) = update.tileset {
            self.tileset = tileset;
        }
        if let Some(background) = update.background {
            self.background = background;
        }
        if let Some(fill) = update.fill {
            self.fill = fill;
        }
        if let Some(fill_paddings) = update.fill_paddings {
            self.fill_paddings = fill_paddings;
        }
        if let Some(progress) = update.progress {
            self.progress = progress;
        }
        if let Some(width) = update.width {
            self.width = width;
        }
        if let Some(height) = update.height {
            self.height = height;
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct TextRender {
    pub text: String,
    pub font_family: String,
    pub size: u32,
    pub letter_spacing: i32,
    pub align: TextRenderAlignment,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum TextRenderAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum RenderKindClean {
    AnimatedSprite,
    Sprite,
    Text,
    ProgressBar,
}
