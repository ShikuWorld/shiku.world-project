use std::fs;
use std::fs::{create_dir_all, remove_dir_all, rename, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;

use log::{debug, error};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::core::blueprint::def::{
    BlueprintError, BlueprintResource, BlueprintService, Chunk, Conductor, FileBrowserFileKind,
    FileBrowserResult, GameMap, GidMap, LayerKind, Module, ResourceKind, ResourceLoaded,
    ResourcePath, Tileset,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::{cantor_pair, get_out_dir, safe_unwrap};

impl Module {
    pub fn new(name: String, id: String) -> Module {
        Module {
            id,
            name,
            max_guests: 0,
            min_guests: 0,
            gid_map: GidMap(Vec::new()),
            exit_points: Vec::new(),
            insert_points: Vec::new(),
            resources: Vec::new(),
            close_after_full: false,
        }
    }
}

impl BlueprintService {
    pub fn create() -> Result<BlueprintService, BlueprintError> {
        Self::setup_blueprints()?;

        Ok(BlueprintService {})
    }

    fn setup_blueprints() -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        create_dir_all(out_dir.join("modules"))?;

        Ok(())
    }

    pub fn browse_directory(directory: String) -> Result<FileBrowserResult, BlueprintError> {
        let browsing_dir = get_out_dir().join(directory.clone());
        let mut file_browser_entry = FileBrowserResult {
            path: browsing_dir.display().to_string(),
            dir: directory.clone(),
            resources: Vec::new(),
            dirs: Vec::new(),
        };
        for entry in WalkDir::new(browsing_dir.clone())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .flatten()
        {
            let file_name = safe_unwrap(entry.file_name().to_str(), BlueprintError::OsParsing)?;
            match Self::determine_file_type(file_name) {
                FileBrowserFileKind::Scene => {

                }
                FileBrowserFileKind::Tileset => {
                    file_browser_entry.resources.push(BlueprintResource {
                        file_name: file_name.to_string(),
                        dir: directory.clone(),
                        path: format!("{}/{}", directory, file_name),
                        kind: ResourceKind::Tileset,
                    });
                }
                FileBrowserFileKind::Map => {
                    file_browser_entry.resources.push(BlueprintResource {
                        file_name: file_name.to_string(),
                        dir: directory.clone(),
                        path: format!("{}/{}", directory, file_name),
                        kind: ResourceKind::Map,
                    });
                }
                FileBrowserFileKind::Folder => {
                    file_browser_entry.dirs.push(file_name.into());
                }
                FileBrowserFileKind::Module => {}
                FileBrowserFileKind::Unknown => {
                    error!("Unknown file type: {}", file_name);
                }
            }
        }
        Ok(file_browser_entry)
    }

    pub fn load_resource_by_path(path: &str) -> ResourceLoaded {
        if let Ok(path_buf) = PathBuf::from_str(path) {
            if let Some(file_name_os) = path_buf.as_path().file_name() {
                if let Some(file_name) = file_name_os.to_str() {
                    return match BlueprintService::determine_resource_type(file_name) {
                        ResourceKind::Scene => ResourceLoaded::Unknown,
                        ResourceKind::Tileset => match Blueprint::load_tileset(path_buf) {
                            Ok(tileset) => ResourceLoaded::Tileset(tileset),
                            Err(err) => {
                                error!("Could not load Resource: {:?}", err);
                                ResourceLoaded::Unknown
                            }
                        },
                        ResourceKind::Map => match Blueprint::load_map(path_buf) {
                            Ok(map) => ResourceLoaded::Map(map),
                            Err(err) => {
                                error!("Could not load Resource: {:?}", err);
                                ResourceLoaded::Unknown
                            }
                        },
                        ResourceKind::Unknown => ResourceLoaded::Unknown,
                    };
                }
            }
        }

        ResourceLoaded::Unknown
    }

    fn determine_resource_type(file_name: &str) -> ResourceKind {
        let parts: Vec<&str> = file_name.split('.').collect();

        match parts.as_slice() {
            [_, "tileset", "json"] => ResourceKind::Tileset,
            [_, "map", "json"] => ResourceKind::Map,
            _ => ResourceKind::Unknown,
        }
    }

    pub fn load_module_tilesets(
        resources: &[BlueprintResource],
    ) -> Result<Vec<Tileset>, BlueprintError> {
        let mut tiles = Vec::new();
        for resource in resources.iter().filter(|r| ResourceKind::Tileset == r.kind) {
            tiles.push(Blueprint::load_tileset(resource.path.clone().into())?);
        }
        Ok(tiles)
    }

    pub fn generate_gid_map(resources: &[BlueprintResource]) -> Result<GidMap, BlueprintError> {
        let mut gid_map = Vec::new();
        let mut current_count = 0;
        debug!("tileset path {:?}", resources);
        for resource in resources.iter().filter(|r| ResourceKind::Tileset == r.kind) {
            let tileset = Blueprint::load_tileset(resource.path.clone().into())?;
            gid_map.push((resource.path.clone(), current_count));
            if tileset.image.is_some() {
                current_count += tileset.tile_count;
            } else {
                current_count += tileset.tiles.len() as u32;
            }
        }
        Ok(GidMap(gid_map))
    }

    fn determine_file_type(file_name: &str) -> FileBrowserFileKind {
        let parts: Vec<&str> = file_name.split('.').collect();

        match parts.as_slice() {
            [_, "tileset", "json"] => FileBrowserFileKind::Tileset,
            [_, "map", "json"] => FileBrowserFileKind::Map,
            [_, "scene", "json"] => FileBrowserFileKind::Scene,
            [_, "json"] => FileBrowserFileKind::Module,
            [_] => FileBrowserFileKind::Folder,
            _ => FileBrowserFileKind::Unknown,
        }
    }

    pub fn module_exists(module_name: &String) -> bool {
        let dir_path = get_out_dir().join("modules").join(module_name);
        let file_path = dir_path.join(format!("{}.json", module_name));
        file_path.exists()
    }

    pub fn change_module_name(module: &mut Module, new_name: String) -> Result<(), BlueprintError> {
        if Self::module_exists(&new_name) {
            return Err(BlueprintError::FileAlreadyExists);
        }
        let old_module_path = get_out_dir().join("modules").join(&module.name);
        let new_module_path = get_out_dir().join("modules").join(&new_name);
        rename(old_module_path, new_module_path)?;
        let old_file_name = get_out_dir()
            .join("modules")
            .join(&new_name)
            .join(format!("{}.json", &module.name));
        let new_file_name = get_out_dir()
            .join("modules")
            .join(&new_name)
            .join(format!("{}.json", &new_name));
        rename(old_file_name, new_file_name)?;
        module.name = new_name;
        Ok(())
    }

    pub fn delete_module(module_name: &String) -> Result<(), BlueprintError> {
        let module_path = get_out_dir().join("modules").join(module_name);
        debug!("Removing {:?}", module_path.to_str());
        remove_dir_all(module_path)?;
        Ok(())
    }

    pub fn create_module(module_name: String) -> Result<Module, BlueprintError> {
        let dir_path = get_out_dir().join("modules").join(&module_name);

        create_dir_all(&dir_path)?;

        let file_path = dir_path.join(format!("{}.json", &module_name));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }

        let module = Module::new(module_name, Uuid::new_v4().to_string());
        Self::save_module(&module)?;

        Ok(module)
    }

    pub fn lazy_load_module(module_name: String) -> Result<Module, BlueprintError> {
        let result = Self::create_module(module_name.clone());
        if let Err(BlueprintError::FileAlreadyExists) = result {
            Self::load_module(module_name)
        } else {
            result
        }
    }

    pub fn get_all_modules() -> Result<Vec<Module>, BlueprintError> {
        let dir_path = get_out_dir().join("modules");
        let paths = fs::read_dir(dir_path)?;
        let mut modules = Vec::new();
        for path in paths {
            let module_name = path?
                .file_name()
                .into_string()
                .unwrap_or("MODULE_NAME_BROKEN".into());
            modules.push(Self::load_module(module_name)?);
        }

        Ok(modules)
    }

    pub fn load_all_maps_for_module(module: &Module) -> Result<Vec<GameMap>, BlueprintError> {
        let out_dir = get_out_dir();
        let mut maps = Vec::new();
        for resource in module
            .resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Map)
        {
            maps.push(Blueprint::load_map(out_dir.join(resource.path.clone()))?);
        }

        Ok(maps)
    }

    pub fn load_module(module_name: String) -> Result<Module, BlueprintError> {
        let dir_path = get_out_dir().join("modules").join(&module_name);
        let file_path = dir_path.join(format!("{}.json", &module_name));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!("{:?}", file_path)));
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save_module(module: &Module) -> Result<(), BlueprintError> {
        let file_path = get_out_dir()
            .join("modules")
            .join(&module.name)
            .join(format!("{}.json", &module.name));
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, module)?)
    }

    pub fn load_conductor_blueprint() -> Result<Conductor, BlueprintError> {
        let file_path = get_out_dir().join("conductor.json");
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist("conductor.json".into()));
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save_conductor_blueprint(blueprint: &Conductor) -> Result<(), BlueprintError> {
        let file_path = get_out_dir().join("conductor.json");
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, blueprint)?)
    }
}

impl GameMap {
    pub fn set_chunk(&mut self, layer_kind: LayerKind, chunk: Chunk) {
        self.terrain
            .entry(layer_kind)
            .or_default()
            .insert(cantor_pair(chunk.position.0, chunk.position.1), chunk);
    }
}
