use crate::core::entity::def::EntityId;
use crate::core::entity::physics::{PhysicalShape, PhysicsType};
use crate::resource_module::map::def::LayerName;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObjectVariant {
    pub start_id: u32,
    pub gids: HashMap<String, u32>,
    pub shapes: HashMap<String, PhysicalShape>,
    pub offset_2d: (f32, f32),
    pub tile_size: (f32, f32),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObjectType {
    pub name: String,
    pub props: Vec<ObjectTypeProp>,
    pub physics: HashMap<String, PhysicsType>,
    pub physics_position: String,
    pub layer_name: LayerName,
    pub render: RenderType,
    pub variants: HashMap<String, ObjectVariant>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ObjectTypeProp {
    pub name: String,
    pub kind: String,
    pub default: PropertyKind,
    pub property_type: PropertyType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PropertyKind {
    #[serde(rename = "float")]
    Float(Real),
    #[serde(rename = "int")]
    Int(i32),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "object")]
    Object(EntityId),
    #[serde(rename = "class")]
    Class(String),
    #[serde(rename = "bool")]
    Boolean(bool),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RenderType {
    StaticImage,
    RenderTypeText,
    RenderTypeTimer,
    NoRender,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PropertyType {
    Physics(PhysicsType),
    Tween,
    Render(RenderType),
    None,
}

impl Display for PropertyKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PropertyKind::Float(_) => "Real".to_string(),
                PropertyKind::Int(_) => "u32".to_string(),
                PropertyKind::String(_) => "String".to_string(),
                PropertyKind::Object(_) => "EntityId".to_string(),
                PropertyKind::Boolean(_) => "bool".to_string(),
                PropertyKind::Class(value) => value.clone(),
                PropertyKind::None => "String".to_string(),
            }
        )
    }
}

impl Display for RenderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for PropertyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::Physics(_) => write!(f, "Physics"),
            PropertyType::Render(_) => write!(f, "Render"),
            PropertyType::Tween => write!(f, "Tween"),
            PropertyType::None => write!(f, "None"),
        }
    }
}
