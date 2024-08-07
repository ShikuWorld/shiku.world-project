use std::collections::HashMap;
use std::convert::Infallible;
use std::ffi::OsString;
use std::hash::Hash;

use crate::core::blueprint::character_animation::CharacterAnimation;
use crate::core::blueprint::ecs::def::Entity;
use crate::core::blueprint::scene::def::{CollisionShape, Scene, Script};
use crate::core::guest::{ModuleEnterSlot, ModuleExitSlot};
use crate::core::module::ModuleName;
use log::error;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;
use walkdir::Error as WalkDirError;

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
    Audio(Audio),
    Font(Font),
    Map(GameMap),
    Script(Script),
    CharacterAnimation(CharacterAnimation),
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
    Audio,
    Font,
    Module,
    Map,
    Tileset,
    Script,
    Scene,
    Folder,
    CharacterAnimation,
    Unknown,
}

#[derive(TS, Debug, Serialize, Deserialize, PartialEq, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ResourceKind {
    Tileset,
    Audio,
    Font,
    Scene,
    Map,
    Script,
    CharacterAnimation,
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
    pub brushes: Vec<TerrainBrush>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Audio {
    pub name: String,
    pub resource_path: ResourcePath,
    pub audio_path: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Font {
    pub name: String,
    pub resource_path: ResourcePath,
    pub font_path: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct StandardKernelThree {
    inside: Gid,

    top_left_corner: Gid,
    top_right_corner: Gid,
    bottom_left_corner: Gid,
    bottom_right_corner: Gid,

    top_left_inner_corner: Gid,
    top_right_inner_corner: Gid,
    bottom_left_inner_corner: Gid,
    bottom_right_inner_corner: Gid,

    top_edge: Gid,
    bottom_edge: Gid,
    left_edge: Gid,
    right_edge: Gid,

    left_top_bottom_right_middle_piece: Gid,
    right_top_bottom_left_middle_piece: Gid,
}

type BrushName = String;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum TerrainBrush {
    StandardKernelThree(BrushName, StandardKernelThree),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct TerrainParams {
    pub chunk_size: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct WorldParams {
    pub terrain_params: TerrainParams,
    pub camera_settings: CameraSettings,
    pub camera_ref: Option<Entity>,
}

pub type LayerParralaxMap = HashMap<LayerKind, (f32, f32)>;

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
pub struct SimpleAnimationFrame {
    id: u32,
    duration: u32,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Default)]
#[ts(export, export_to = "blueprints/")]
pub struct Tile {
    pub id: u32,
    pub image: Option<Image>,
    pub animation: Option<Vec<SimpleAnimationFrame>>,
    pub collision_shape: Option<CollisionShape>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Chunk {
    pub position: (i32, i32),
    pub data: Vec<Gid>,
}

impl Chunk {
    pub fn new(position: (i32, i32), chunk_size: usize) -> Chunk {
        Chunk {
            position,
            data: vec![0; chunk_size * chunk_size],
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct ChunkUpdate {
    pub position: (i32, i32),
    pub tile_updates: HashMap<i32, HashMap<i32, Gid>>,
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
pub struct CharAnimationToTilesetMap(pub HashMap<ResourcePath, ResourcePath>);

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Module {
    pub id: ModuleId,
    pub name: ModuleName,
    pub resources: Vec<BlueprintResource>,
    pub main_map: Option<ResourcePath>,
    pub gid_map: GidMap,
    pub char_animation_to_tileset_map: CharAnimationToTilesetMap,
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
    pub main_map: Option<Option<ResourcePath>>,
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
            main_map: None,
            max_guests: None,
            min_guests: None,
        }
    }

    pub fn resources(mut self, resources: Vec<BlueprintResource>) -> ModuleUpdate {
        self.resources = Some(resources);
        self
    }
}

pub trait JsonResource {
    fn get_resource_extension() -> &'static str;
    fn get_resource_kind(&self) -> ResourceKind;
    fn get_full_resource_path(&self) -> String {
        format!(
            "{}/{}.{}.json",
            self.get_resource_dir(),
            self.get_name(),
            Self::get_resource_extension()
        )
    }
    fn get_file_name(&self) -> String {
        format!(
            "{}.{}.json",
            self.get_name(),
            Self::get_resource_extension()
        )
    }
    fn get_name(&self) -> &str;
    fn get_resource_dir(&self) -> &str;
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
    pub camera_settings: CameraSettings,
    pub terrain: HashMap<LayerKind, HashMap<u32, Chunk>>,
    pub layer_parallax: HashMap<LayerKind, (f32, f32)>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct MapUpdate {
    pub name: String,
    pub resource_path: ResourcePath,
    pub chunk: Option<(LayerKind, ChunkUpdate)>,
    pub scene: Option<ResourcePath>,
    pub layer_parallax: Option<(LayerKind, (f32, f32))>,
    pub camera_settings: Option<CameraSettings>,
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

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CameraSettings {
    pub(crate) zoom: Real,
    pub(crate) bounds: Option<((Real, Real), (Real, Real))>,
}
