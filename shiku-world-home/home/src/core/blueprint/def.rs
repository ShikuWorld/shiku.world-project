use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::hash::Hash;

use log::error;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use walkdir::Error as WalkDirError;

use crate::core::blueprint::scene::def::{CollisionShape, Scene, Script};
use crate::core::guest::{ModuleEnterSlot, ModuleExitSlot};
use crate::core::module::ModuleName;

pub type EntityId = usize;
pub type JointId = usize;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Conductor {
    pub(crate) module_connection_map: HashMap<ModuleExitSlot, (ModuleId, ModuleEnterSlot)>,
    pub(crate) resources: Vec<BlueprintResource>,
    pub(crate) gid_map: GidMap,
}

impl Conductor {
    pub fn default() -> Conductor {
        Conductor {
            module_connection_map: HashMap::new(),
            resources: Vec::new(),
            gid_map: GidMap(Vec::new()),
        }
    }
}

pub enum ResourceLoaded {
    Tileset(Tileset),
    Scene(Scene),
    Map(GameMap),
    Script(Script),
    Unknown,
}

pub type ResourcePath = String;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct FileBrowserResult {
    pub path: String,
    pub dir: String,
    pub dirs: Vec<String>,
    pub resources: Vec<BlueprintResource>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum FileBrowserFileKind {
    Conductor,
    Module,
    Map,
    Tileset,
    Script,
    Scene,
    Folder,
    Unknown,
}

#[derive(TS, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ResourceKind {
    Tileset,
    Scene,
    Map,
    Script,
    Unknown,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct BlueprintResource {
    pub file_name: String,
    pub dir: ResourcePath,
    pub path: ResourcePath,
    pub kind: ResourceKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Image {
    path: String,
    width: u32,
    height: u32,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Tileset {
    pub name: String,
    pub resource_path: ResourcePath,
    pub image: Option<Image>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub tiles: HashMap<u32, Tile>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct TerrainParams {
    pub chunk_size: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

impl Tileset {
    pub fn get_image_paths(&self) -> Vec<ResourcePath> {
        if let Some(image) = &self.image {
            vec![image.path.clone()]
        } else {
            self.tiles
                .values()
                .filter_map(|t| t.image.as_ref().map(|image| image.path.clone()))
                .collect()
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct AnimationFrame {
    id: u32,
    duration: u32,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Default)]
#[ts(export, export_to = "blueprints/")]
pub struct Tile {
    pub id: u32,
    pub image: Option<Image>,
    pub animation: Option<Vec<AnimationFrame>>,
    pub collision_shape: Option<CollisionShape>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Chunk {
    pub position: (i32, i32),
    pub data: Vec<u32>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct IOPoint {
    pub name: String,
    pub condition_script: String,
}

pub type ModuleId = String;

pub type Gid = u32;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct GidMap(pub Vec<(ResourcePath, Gid)>);

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Module {
    pub id: ModuleId,
    pub name: ModuleName,
    pub resources: Vec<BlueprintResource>,
    pub gid_map: GidMap,
    pub insert_points: Vec<IOPoint>,
    pub exit_points: Vec<IOPoint>,
    pub max_guests: usize,
    pub min_guests: usize,
    pub close_after_full: bool,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct ModuleUpdate {
    pub name: Option<String>,
    pub resources: Option<Vec<BlueprintResource>>,
    pub insert_points: Option<Vec<IOPoint>>,
    pub exit_points: Option<Vec<IOPoint>>,
    pub max_guests: Option<usize>,
    pub min_guests: Option<usize>,
}

impl ModuleUpdate {
    pub fn default() -> ModuleUpdate {
        ModuleUpdate {
            name: None,
            resources: None,
            insert_points: None,
            exit_points: None,
            max_guests: None,
            min_guests: None,
        }
    }

    pub fn resources(mut self, resources: Vec<BlueprintResource>) -> ModuleUpdate {
        self.resources = Some(resources);
        self
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct GameMap {
    pub module_id: String,
    pub world_id: String,
    pub name: String,
    pub resource_path: String,
    pub chunk_size: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub main_scene: ResourcePath,
    pub terrain: HashMap<LayerKind, HashMap<u32, Chunk>>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct MapUpdate {
    pub name: String,
    pub resource_path: ResourcePath,
    pub chunk: Option<(LayerKind, Chunk)>,
    pub scene: Option<ResourcePath>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Layer {
    pub name: String,
    pub kind: LayerKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[ts(export, export_to = "blueprints/")]
pub enum LayerKind {
    BG10,
    BG09,
    BG08,
    BG07,
    BG06,
    BG05,
    BG04,
    BG03,
    BG02,
    BG01,
    BG00,
    ObjectsBelow,
    Terrain,
    ObjectsFront,
    FG00,
    FG01,
    FG02,
    FG03,
    FG04,
    FG05,
    FG06,
    FG07,
    FG08,
    FG09,
    FG10,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Joint {
    id: JointId,
    entity_a: EntityId,
    entity_b: EntityId,
}

pub struct BlueprintService;

#[derive(Error, Debug)]
pub enum BlueprintError {
    #[error("Tried to load a file that should exist")]
    FileDoesNotExist(String),
    #[error("Tried to write a file that shouldn't already exist")]
    FileAlreadyExists,
    #[error("Could not load blueprint due to IO error.")]
    IOError(#[from] std::io::Error),
    #[error("Lock Poisoned while writing.")]
    WritePoison(&'static str),
    #[error("Lock Poisoned while reading.")]
    ReadPoison(&'static str),
    #[error("Could not load blueprint due to malformed json.")]
    SerdeJSON(#[from] serde_json::error::Error),
    #[error("Impossible error")]
    Impossible(#[from] Infallible),
    #[error("Not able to access nested object")]
    AccessNested(&'static str),
    #[error("Failed to convert to String from OsString")]
    ConversionToString(OsString),
    #[error("Failed to convert to PathBuf to str")]
    ConversionToStr,
    #[error("File Browsing error")]
    FileBrowsing(#[from] WalkDirError),
    #[error("OS String parsing error")]
    OsParsing,
}
