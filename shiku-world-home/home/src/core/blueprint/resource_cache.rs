use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::RwLock;

use log::{debug, error};
use walkdir::WalkDir;

use crate::core::{get_out_dir, safe_unwrap};
use crate::core::blueprint::def::{BlueprintError, BlueprintService, FileBrowserFileKind, GameMap, Module, ResourcePath, Scene, Tileset};
use crate::core::blueprint::resource_loader::Blueprint;

pub struct ResourceCache {
    pub tilesets: RwLock<HashMap<ResourcePath, Tileset>>,
    pub maps: RwLock<HashMap<ResourcePath, GameMap>>,
    pub scenes: RwLock<HashMap<ResourcePath, Scene>>,
    pub modules: RwLock<HashMap<ResourcePath, Module>>
}

static RESOURCE_CACHE: OnceLock<ResourceCache> = OnceLock::new();

pub fn get_resource_cache() -> &'static ResourceCache {
    RESOURCE_CACHE.get_or_init(|| {
        ResourceCache {
            tilesets: RwLock::new(HashMap::new()),
            maps: RwLock::new(HashMap::new()),
            scenes: RwLock::new(HashMap::new()),
            modules: RwLock::new(HashMap::new())
        }
    })
}

pub fn init_resource_cache() -> Result<(), BlueprintError> {
    let resources = get_resource_cache();
    let root = get_out_dir();
    debug!("Root folder for resources is {:?}", root);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let full_resource_path = entry.path();
        let file_name = safe_unwrap(entry.file_name().to_str(), BlueprintError::OsParsing)?;
        match BlueprintService::determine_file_type(file_name) {
            FileBrowserFileKind::Scene => {
                let scene = Blueprint::load_from_file(PathBuf::from(full_resource_path))?;
                resources.scenes.write().map_err(|_| BlueprintError::WritePoison("Write cache fail. Poison?!"))?.insert(full_resource_path.display().to_string(), scene);
                debug!("Successfully loaded {:?}", full_resource_path.display());
            }
            FileBrowserFileKind::Tileset => {
                let tileset = Blueprint::load_from_file(PathBuf::from(full_resource_path))?;
                resources.tilesets.write().map_err(|_| BlueprintError::WritePoison("Write cache fail. Poison?!"))?.insert(full_resource_path.display().to_string(), tileset);
                debug!("Successfully loaded {:?}", full_resource_path.display());
            }
            FileBrowserFileKind::Map => {
                let map = Blueprint::load_from_file(PathBuf::from(full_resource_path))?;
                resources.maps.write().map_err(|_| BlueprintError::WritePoison("Write cache fail. Poison?!"))?.insert(full_resource_path.display().to_string(), map);
                debug!("Successfully loaded {:?}", full_resource_path.display());
            }
            FileBrowserFileKind::Module => {
                let module = Blueprint::load_from_file(PathBuf::from(full_resource_path))?;
                resources.modules.write().map_err(|_| BlueprintError::WritePoison("Write cache fail. Poison?!"))?.insert(full_resource_path.display().to_string(), module);
                debug!("Successfully loaded {:?}", full_resource_path.display());
            }
            FileBrowserFileKind::Folder | FileBrowserFileKind::Conductor => {

            }
            FileBrowserFileKind::Unknown => {
                error!("Unknown file type: {}", file_name);
            }
        }
    }

    Ok(())
}


