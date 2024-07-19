use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;

use log::{debug, error};
use walkdir::WalkDir;

use crate::core::blueprint::character_animation::CharacterAnimation;
use crate::core::blueprint::def::{
    BlueprintError, BlueprintResource, BlueprintService, CharAnimationToTilesetMap, Chunk,
    ChunkUpdate, Conductor, FileBrowserFileKind, FileBrowserResult, GameMap, Gid, GidMap,
    JsonResource, LayerKind, MapUpdate, Module, ResourceKind, ResourceLoaded, Tileset,
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
            char_animation_to_tileset_map: CharAnimationToTilesetMap(HashMap::new()),
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
                | FileBrowserFileKind::CharacterAnimation
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
                    let result = match BlueprintService::determine_resource_type(file_name) {
                        ResourceKind::Scene => {
                            Blueprint::load_scene(path_buf).map(ResourceLoaded::Scene)
                        }
                        ResourceKind::CharacterAnimation => {
                            Blueprint::load_character_animation(path_buf)
                                .map(ResourceLoaded::CharacterAnimation)
                        }
                        ResourceKind::Tileset => {
                            Blueprint::load_tileset(path_buf).map(ResourceLoaded::Tileset)
                        }
                        ResourceKind::Map => Blueprint::load_map(path_buf).map(ResourceLoaded::Map),
                        ResourceKind::Script => {
                            Blueprint::load_script(path_buf).map(ResourceLoaded::Script)
                        }
                        ResourceKind::Unknown => Ok(ResourceLoaded::Unknown),
                    };
                    return result.unwrap_or_else(|err| {
                        error!("Could not load Resource: {:?}", err);
                        ResourceLoaded::Unknown
                    });
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
            [_, "char_anim", "json"] => ResourceKind::CharacterAnimation,
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

    pub fn generate_gid_and_char_anim_to_tileset_map(
        resources: &[BlueprintResource],
    ) -> Result<(GidMap, CharAnimationToTilesetMap), BlueprintError> {
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
        let char_animation_to_tileset_map =
            BlueprintService::generate_character_animation_to_tileset_map(resources)?;
        Ok((GidMap(gid_map), char_animation_to_tileset_map))
    }

    pub fn generate_character_animation_to_tileset_map(
        resources: &[BlueprintResource],
    ) -> Result<CharAnimationToTilesetMap, BlueprintError> {
        let mut animation_to_tileset_map = HashMap::new();
        for resource in resources
            .iter()
            .filter(|r| ResourceKind::CharacterAnimation == r.kind)
        {
            debug!("character animation path {:?}", resources);
            let character_animation =
                Blueprint::load_character_animation(resource.path.clone().into())?;
            animation_to_tileset_map
                .insert(resource.path.clone(), character_animation.tileset_resource);
        }
        Ok(CharAnimationToTilesetMap(animation_to_tileset_map))
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
            [_, "char_anim", "json"] => FileBrowserFileKind::CharacterAnimation,
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
            FileBrowserFileKind::CharacterAnimation => ResourceKind::CharacterAnimation,
            FileBrowserFileKind::Map => ResourceKind::Map,
            FileBrowserFileKind::Tileset => ResourceKind::Tileset,
            FileBrowserFileKind::Script => ResourceKind::Script,
            FileBrowserFileKind::Scene => ResourceKind::Scene,
        }
    }
}

impl GameMap {
    pub fn apply_chunk_update(
        &mut self,
        layer_kind: LayerKind,
        chunk_update: ChunkUpdate,
    ) -> Option<Chunk> {
        let chunk_id = cantor_pair(chunk_update.position.0, chunk_update.position.1);
        let chunk = self
            .terrain
            .entry(layer_kind.clone())
            .or_default()
            .entry(chunk_id)
            .or_insert_with(|| Chunk::new(chunk_update.position, self.chunk_size as usize));

        let mut has_updated = false;
        for (y, tile) in chunk_update.tile_updates {
            for (x, gid) in tile {
                let i = y * self.chunk_size as i32 + x;
                if (chunk.data.len() as i32) > i && i >= 0 {
                    if chunk.data[i as usize] != gid {
                        chunk.data[i as usize] = gid;
                        has_updated = true;
                    }
                } else {
                    error!("Invalid index for chunk insertion!!!: {}", i);
                }
            }
        }

        has_updated.then_some(chunk.clone())
    }
}

impl JsonResource for Module {
    fn get_resource_extension() -> &'static str {
        "module"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Unknown
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        self.name.as_str()
    }
}

impl JsonResource for GameMap {
    fn get_resource_extension() -> &'static str {
        "map"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Map
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl JsonResource for Script {
    fn get_resource_extension() -> &'static str {
        "script"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Script
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl JsonResource for Scene {
    fn get_resource_extension() -> &'static str {
        "scene"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Scene
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl JsonResource for Tileset {
    fn get_resource_extension() -> &'static str {
        "tileset"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Tileset
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl JsonResource for MapUpdate {
    fn get_resource_extension() -> &'static str {
        "map"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::Map
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl JsonResource for CharacterAnimation {
    fn get_resource_extension() -> &'static str {
        "char_anim"
    }
    fn get_resource_kind(&self) -> ResourceKind {
        ResourceKind::CharacterAnimation
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn get_resource_dir(&self) -> &str {
        &self.resource_path
    }
}

impl<T: JsonResource> From<&T> for BlueprintResource {
    fn from(value: &T) -> Self {
        BlueprintResource {
            file_name: value.get_file_name(),
            dir: value.get_resource_dir().into(),
            path: value.get_full_resource_path(),
            kind: value.get_resource_kind(),
        }
    }
}
