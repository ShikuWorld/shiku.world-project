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
    tilesets: RwLock<HashMap<ResourcePath, Tileset>>,
    maps: RwLock<HashMap<ResourcePath, GameMap>>,
    scenes: RwLock<HashMap<ResourcePath, Scene>>,
    modules: RwLock<HashMap<ResourcePath, Module>>
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
    let mut maps = resources.maps.write().map_err(|_| BlueprintError::WritePoison)?;
    let mut tilesets = resources.tilesets.write().map_err(|_| BlueprintError::WritePoison)?;
    let mut scenes = resources.scenes.write().map_err(|_| BlueprintError::WritePoison)?;
    let mut modules = resources.modules.write().map_err(|_| BlueprintError::WritePoison)?;
    let root = get_out_dir();
    debug!("{:?}", root);
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let full_resource_path = entry.path();
        let file_name = safe_unwrap(entry.file_name().to_str(), BlueprintError::OsParsing)?;
        debug!("{:?}", full_resource_path.display());
        match BlueprintService::determine_file_type(file_name) {
            FileBrowserFileKind::Scene => {
                let scene = Blueprint::load_scene(PathBuf::from(full_resource_path))?;
                scenes.insert(full_resource_path.display().to_string(), scene);
            }
            FileBrowserFileKind::Tileset => {
                let tileset = Blueprint::load_tileset(PathBuf::from(full_resource_path))?;
                tilesets.insert(full_resource_path.display().to_string(), tileset);
            }
            FileBrowserFileKind::Map => {
                let map = Blueprint::load_map(PathBuf::from(full_resource_path))?;
                maps.insert(full_resource_path.display().to_string(), map);
            }
            FileBrowserFileKind::Module => {
                debug!("Mh module loading?");
            }
            FileBrowserFileKind::Folder => {

            }
            FileBrowserFileKind::Unknown => {
                error!("Unknown file type: {}", file_name);
            }
        }
    }

    Ok(())
}
