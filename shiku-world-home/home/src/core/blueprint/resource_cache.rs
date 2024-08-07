use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::RwLock;

use log::{debug, error};
use serde::de::DeserializeOwned;
use walkdir::WalkDir;

use crate::core::blueprint::character_animation::CharacterAnimation;
use crate::core::blueprint::def::{
    Audio, BlueprintError, BlueprintService, FileBrowserFileKind, Font, GameMap, JsonResource,
    Module, ResourcePath, Tileset,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{Scene, Script};
use crate::core::{get_out_dir, safe_unwrap};

pub struct ResourceCache {
    pub tilesets: RwLock<HashMap<ResourcePath, Tileset>>,
    pub maps: RwLock<HashMap<ResourcePath, GameMap>>,
    pub scenes: RwLock<HashMap<ResourcePath, Scene>>,
    pub scripts: RwLock<HashMap<ResourcePath, Script>>,
    pub character_animations: RwLock<HashMap<ResourcePath, CharacterAnimation>>,
    pub modules: RwLock<HashMap<ResourcePath, Module>>,
    pub fonts: RwLock<HashMap<ResourcePath, Font>>,
    pub audios: RwLock<HashMap<ResourcePath, Audio>>,
}

static RESOURCE_CACHE: OnceLock<ResourceCache> = OnceLock::new();

pub fn get_resource_cache() -> &'static ResourceCache {
    RESOURCE_CACHE.get_or_init(|| ResourceCache {
        tilesets: RwLock::new(HashMap::new()),
        maps: RwLock::new(HashMap::new()),
        scenes: RwLock::new(HashMap::new()),
        scripts: RwLock::new(HashMap::new()),
        character_animations: RwLock::new(HashMap::new()),
        modules: RwLock::new(HashMap::new()),
        fonts: RwLock::new(HashMap::new()),
        audios: RwLock::new(HashMap::new()),
    })
}

pub fn load_to_map<T>(
    path: &Path,
    map: &RwLock<HashMap<ResourcePath, T>>,
) -> Result<(), BlueprintError>
where
    T: DeserializeOwned,
{
    let audio = Blueprint::load_from_file(PathBuf::from(path))?;
    map.write()
        .map_err(|_| BlueprintError::WritePoison("Write cache fail. Poison?!"))?
        .insert(path.display().to_string(), audio);
    debug!("Successfully loaded {:?}", path.display());
    Ok(())
}

pub fn init_resource_cache() -> Result<(), BlueprintError> {
    let resources = get_resource_cache();
    let root = get_out_dir();
    debug!("Root folder for resources is {:?}", root);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let full_resource_path = entry.path();
        let file_name = safe_unwrap(entry.file_name().to_str(), BlueprintError::OsParsing)?;
        match BlueprintService::determine_file_type(file_name) {
            FileBrowserFileKind::Audio => {
                load_to_map(full_resource_path, &resources.audios)?;
            }
            FileBrowserFileKind::Font => {
                load_to_map(full_resource_path, &resources.fonts)?;
            }
            FileBrowserFileKind::Scene => {
                load_to_map(full_resource_path, &resources.scenes)?;
            }
            FileBrowserFileKind::Tileset => {
                load_to_map(full_resource_path, &resources.tilesets)?;
            }
            FileBrowserFileKind::Map => {
                load_to_map(full_resource_path, &resources.maps)?;
            }
            FileBrowserFileKind::Module => {
                load_to_map(full_resource_path, &resources.modules)?;
            }
            FileBrowserFileKind::Script => {
                load_to_map(full_resource_path, &resources.scripts)?;
            }
            FileBrowserFileKind::CharacterAnimation => {
                load_to_map(full_resource_path, &resources.character_animations)?;
            }
            FileBrowserFileKind::Folder | FileBrowserFileKind::Conductor => {}
            FileBrowserFileKind::Unknown => {
                error!("Unknown file type: {}", file_name);
            }
        }
    }

    Ok(())
}
