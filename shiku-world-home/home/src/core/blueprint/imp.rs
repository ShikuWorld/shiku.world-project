use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;

use log::{debug, error};
use walkdir::WalkDir;

use crate::core::blueprint::def::{
    BlueprintError, BlueprintResource, BlueprintService, Chunk, Conductor, FileBrowserFileKind,
    FileBrowserResult, GameMap, Gid, GidMap, LayerKind, Module, ResourceKind, ResourceLoaded,
    Tileset,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{CollisionShape, Scene, Script};
use crate::core::{cantor_pair, get_out_dir, safe_unwrap};

impl Module {
    pub fn new(name: String, id: String) -> Module {
        Module {
            id,
            name,
            main_map: None,
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
            let file_browser_file_type = Self::determine_file_type(file_name);
            match &file_browser_file_type {
                FileBrowserFileKind::Scene
                | FileBrowserFileKind::Tileset
                | FileBrowserFileKind::Map
                | FileBrowserFileKind::Script => {
                    file_browser_entry.resources.push(BlueprintResource {
                        file_name: file_name.to_string(),
                        dir: directory.clone(),
                        path: format!("{}/{}", directory, file_name),
                        kind: file_browser_file_type.into(),
                    });
                }
                FileBrowserFileKind::Folder => {
                    file_browser_entry.dirs.push(file_name.into());
                }
                FileBrowserFileKind::Module | FileBrowserFileKind::Conductor => {}
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
                        ResourceKind::Scene => match Blueprint::load_scene(path_buf) {
                            Ok(scene) => ResourceLoaded::Scene(scene),
                            Err(err) => {
                                error!("Could not load Resource: {:?}", err);
                                ResourceLoaded::Unknown
                            }
                        },
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
                        ResourceKind::Script => match Blueprint::load_script(path_buf) {
                            Ok(script) => ResourceLoaded::Script(script),
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
            [_, "scene", "json"] => ResourceKind::Scene,
            [_, "script", "json"] => ResourceKind::Script,
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
        for resource in resources.iter().filter(|r| ResourceKind::Tileset == r.kind) {
            debug!("tileset path {:?}", resources);
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

    pub fn generate_gid_to_shape_map(
        resources: &[BlueprintResource],
    ) -> Result<HashMap<Gid, CollisionShape>, BlueprintError> {
        let mut gid_to_collision_shape_map = HashMap::new();
        let mut current_count = 0;
        for resource in resources.iter().filter(|r| ResourceKind::Tileset == r.kind) {
            let tileset = Blueprint::load_tileset(resource.path.clone().into())?;

            for (id, t) in &tileset.tiles {
                if let Some(c) = &t.collision_shape {
                    gid_to_collision_shape_map.insert(current_count + id, c.clone());
                }
            }

            if tileset.image.is_some() {
                current_count += tileset.tile_count;
            } else {
                current_count += tileset.tiles.len() as u32;
            }
        }
        Ok(gid_to_collision_shape_map)
    }

    pub fn determine_file_type(file_name: &str) -> FileBrowserFileKind {
        let parts: Vec<&str> = file_name.split('.').collect();

        match parts.as_slice() {
            ["conductor", "json"] => FileBrowserFileKind::Conductor,
            [_, "tileset", "json"] => FileBrowserFileKind::Tileset,
            [_, "map", "json"] => FileBrowserFileKind::Map,
            [_, "scene", "json"] => FileBrowserFileKind::Scene,
            [_, "script", "json"] => FileBrowserFileKind::Script,
            [_, "module", "json"] => FileBrowserFileKind::Module,
            [_] => FileBrowserFileKind::Folder,
            _ => FileBrowserFileKind::Unknown,
        }
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

impl From<FileBrowserFileKind> for ResourceKind {
    fn from(value: FileBrowserFileKind) -> Self {
        match value {
            FileBrowserFileKind::Folder
            | FileBrowserFileKind::Unknown
            | FileBrowserFileKind::Conductor
            | FileBrowserFileKind::Module => ResourceKind::Unknown,
            FileBrowserFileKind::Map => ResourceKind::Map,
            FileBrowserFileKind::Tileset => ResourceKind::Tileset,
            FileBrowserFileKind::Script => ResourceKind::Script,
            FileBrowserFileKind::Scene => ResourceKind::Scene,
        }
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

impl From<&Script> for BlueprintResource {
    fn from(value: &Script) -> Self {
        BlueprintResource {
            file_name: format!("{}.script.json", value.name),
            dir: value.resource_path.clone(),
            path: format!("{}/{}.script.json", value.resource_path, value.name),
            kind: ResourceKind::Script,
        }
    }
}

impl From<&Scene> for BlueprintResource {
    fn from(scene: &Scene) -> Self {
        BlueprintResource {
            file_name: format!("{}.scene.json", scene.name),
            dir: scene.resource_path.clone(),
            path: format!("{}/{}.scene.json", scene.resource_path, scene.name),
            kind: ResourceKind::Scene,
        }
    }
}

impl From<&GameMap> for BlueprintResource {
    fn from(value: &GameMap) -> Self {
        BlueprintResource {
            file_name: format!("{}.map.json", value.name),
            dir: value.resource_path.clone(),
            path: format!("{}/{}.map.json", value.resource_path, value.name),
            kind: ResourceKind::Map,
        }
    }
}

impl From<&Tileset> for BlueprintResource {
    fn from(value: &Tileset) -> Self {
        BlueprintResource {
            file_name: format!("{}.tileset.json", value.name),
            dir: value.resource_path.clone(),
            path: format!("{}/{}.tileset.json", value.resource_path, value.name),
            kind: ResourceKind::Tileset,
        }
    }
}
