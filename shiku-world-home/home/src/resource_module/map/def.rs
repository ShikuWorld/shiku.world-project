use crate::core::entity::def::EntityId;
use crate::core::tween::Tween;
use rapier2d::prelude::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tiled::Tileset;
use ts_rs::TS;

#[derive(Debug)]
pub struct TiledMap {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tilesets: Vec<TilesetEntry>,
    pub object_groups: Vec<ObjectGroup>,
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone)]
pub struct TilesetEntry {
    pub first_gid: u32,
    pub path: String,
    pub tileset: Option<Tileset>,
}

pub type ObjectId = String;

#[derive(Debug, Clone)]
pub struct ObjectGroup {
    pub id: String,
    pub layer_name: LayerName,
    pub objects: Vec<GeneralObject>,
}

#[derive(Debug, Clone)]
pub struct ObjectText {
    pub text: String,
    pub font_family: String,
    pub pixel_size: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct GeneralObject {
    pub id: ObjectId,
    pub name: String,
    pub graphic_id: String,
    pub entity_id: String,
    pub x: Real,
    pub y: Real,
    pub width: Real,
    pub height: Real,
    pub kind: String,
    pub text: Option<ObjectText>,
    pub custom_props: HashMap<String, CustomPropType>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CustomPropType {
    #[serde(rename = "float")]
    Float(Real),
    #[serde(rename = "bool")]
    Boolean(bool),
    #[serde(rename = "int")]
    Int(u32),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "object")]
    Object(EntityId),
    Tween(Tween),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[ts(export)]
pub enum LayerName {
    Terrain,
    Empty,
    GameObjects,
    Guest,
    BG0,
    BG1,
    BG2,
    BG3,
    BG4,
    BG5,
    BG6,
    BG7,
    BG8,
    BG9,
    BG10,
    BG11,
    FG0,
    FG1,
    FG2,
    FG3,
    FG4,
    FG5,
    FG6,
    FG7,
    FG8,
    FG9,
    FG10,
    FG11,
    Menu,
}

impl Display for LayerName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: String,
    pub name: LayerName,
    pub encoding: String,
    pub parallax: (Real, Real),
    pub terrain_chunks: HashMap<(i32, i32), TerrainChunk>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct TerrainChunk {
    pub x: Real,
    pub y: Real,
    pub width: Real,
    pub height: Real,
    pub tile_ids: Vec<Vec<u32>>,
    pub layer: LayerName,
}

impl TerrainChunk {
    pub fn new() -> TerrainChunk {
        TerrainChunk {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            tile_ids: vec![],
            layer: LayerName::Empty,
        }
    }
}

impl Layer {
    pub fn new() -> Layer {
        Layer {
            id: "".to_string(),
            name: LayerName::Empty,
            encoding: "".to_string(),
            parallax: (1.0, 1.0),
            terrain_chunks: HashMap::new(),
        }
    }
}

impl TilesetEntry {
    pub fn new() -> TilesetEntry {
        TilesetEntry {
            first_gid: 0,
            path: String::new(),
            tileset: None,
        }
    }
}

impl ObjectGroup {
    pub fn new() -> ObjectGroup {
        ObjectGroup {
            id: "".to_string(),
            layer_name: LayerName::Terrain,
            objects: vec![],
        }
    }
}

impl ObjectText {
    pub fn new() -> ObjectText {
        ObjectText {
            pixel_size: String::new(),
            color: String::new(),
            text: String::new(),
            font_family: String::new(),
        }
    }
}

impl GeneralObject {
    pub fn new() -> GeneralObject {
        GeneralObject {
            id: String::new(),
            name: String::new(),
            graphic_id: String::new(),
            kind: String::new(),
            y: 0.0,
            x: 0.0,
            width: 0.0,
            height: 0.0,
            text: None,
            custom_props: HashMap::new(),
            entity_id: String::new(),
        }
    }

    pub fn get_custom_prop_tween(&self, key: &str) -> Tween {
        let mut object_value = Tween::new();

        if let Some(CustomPropType::Tween(tween)) = self.custom_props.get(key) {
            object_value = tween.clone();
        }

        object_value
    }

    pub fn get_custom_prop_entity_id(&self, key: &str) -> EntityId {
        let mut object_value = "NOT_FOUND".to_string();

        if let Some(CustomPropType::Object(entity_id)) = self.custom_props.get(key) {
            object_value = entity_id.clone();
        }

        object_value
    }

    pub fn get_custom_prop_real(&self, key: &str) -> Real {
        let mut float_value = 0.0;

        if let Some(CustomPropType::Float(float)) = self.custom_props.get(key) {
            float_value = *float;
        }

        float_value
    }

    pub fn get_custom_prop_u_32(&self, key: &str) -> u32 {
        let mut int_value = 0;

        if let Some(CustomPropType::Int(int)) = self.custom_props.get(key) {
            int_value = *int;
        }

        int_value
    }

    pub fn get_custom_prop_bool(&self, key: &str) -> bool {
        let mut bool_value = false;

        if let Some(CustomPropType::Boolean(bool)) = self.custom_props.get(key) {
            bool_value = *bool;
        }

        bool_value
    }

    pub fn get_custom_prop_string(&self, key: &str) -> String {
        let mut string_value = "".to_string();

        if let Some(CustomPropType::String(string)) = self.custom_props.get(key) {
            string_value = string.clone();
        }

        string_value
    }
}
