use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, remove_dir_all, remove_file, rename, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::RwLock;

use crate::core::blueprint::character_animation::CharacterAnimation;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::core::blueprint::def::{
    BlueprintError, GameMap, IOPoint, JsonResource, Module, ResourcePath, Tileset,
};
use crate::core::blueprint::ecs::def::{EntityUpdate, EntityUpdateKind};
use crate::core::blueprint::resource_cache::get_resource_cache;
use crate::core::blueprint::scene::def::{GameNodeKind, Scene, Script};
use crate::core::get_out_dir;
use crate::core::module::ModuleName;

pub struct Blueprint;

impl Blueprint {
    pub fn create<T: Serialize + Clone + JsonResource>(
        resource: &T,
        resource_path: &str,
        resource_name: &str,
        resource_map: &RwLock<HashMap<ResourcePath, T>>,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path)?;
        let directory_path = out_dir.join(resource_path);
        create_dir_all(directory_path.as_path())?;
        let file_path = directory_path.join(format!(
            "{}.{}.json",
            resource_name,
            T::get_resource_extension()
        ));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }
        let file = File::create(file_path.clone())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, resource)?;
        let file_path_as_string = Self::path_buf_to_string(file_path)?;
        resource_map
            .write()
            .map_err(|_| {
                BlueprintError::WritePoison("Could not add resource to cache due to poison.")
            })?
            .insert(file_path_as_string, resource.clone());
        Ok(())
    }

    pub fn load<T: DeserializeOwned + Clone + JsonResource>(
        path: PathBuf,
        resource_map: &RwLock<HashMap<ResourcePath, T>>,
    ) -> Result<T, BlueprintError> {
        let actual_path_buf = get_out_dir().join(path);
        let actual_path = Self::path_buf_to_string(actual_path_buf)?;
        resource_map
            .read()
            .map_err(|_| BlueprintError::ReadPoison("Could not load resource due to poison."))?
            .get(&actual_path)
            .cloned()
            .ok_or(BlueprintError::FileDoesNotExist(actual_path.to_string()))
    }
    pub fn load_from_file<T: DeserializeOwned>(path: PathBuf) -> Result<T, BlueprintError> {
        debug!("Loading {:?}", path.display());
        let actual_path = get_out_dir().join(path);
        if !actual_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!(
                "load {:?}",
                actual_path
            )));
        }
        let file = File::open(actual_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save<T: Serialize + Clone + JsonResource>(
        resource: &T,
        resource_path: &str,
        resource_name: &str,
        resource_map: &RwLock<HashMap<ResourcePath, T>>,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path)?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!(
            "{}.{}.json",
            resource_name,
            T::get_resource_extension()
        ));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!("{:?}", file_path)));
        }
        let file = File::create(file_path.clone())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, resource)?;
        let file_path_as_string = Self::path_buf_to_string(file_path)?;
        resource_map
            .write()
            .map_err(|_| {
                BlueprintError::WritePoison("Could not add resource to cache due to poison.")
            })?
            .insert(file_path_as_string, resource.clone());
        Ok(())
    }

    pub fn delete<T: JsonResource>(
        resource_path: &str,
        resource_name: &str,
        resource_map: &RwLock<HashMap<ResourcePath, T>>,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path)?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!(
            "{}.{}.json",
            resource_name,
            T::get_resource_extension()
        ));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!("{:?}", file_path)));
        }
        remove_file(file_path.clone())?;
        let file_path_as_string = Self::path_buf_to_string(file_path)?;
        resource_map
            .write()
            .map_err(|_| {
                BlueprintError::WritePoison("Could not remove resource from cache due to poison.")
            })?
            .remove(&file_path_as_string);
        Ok(())
    }

    pub fn create_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::create(
            tileset,
            &tileset.resource_path,
            &tileset.name,
            &resources.tilesets,
        )
    }

    pub fn load_tileset(path: PathBuf) -> Result<Tileset, BlueprintError> {
        debug!("Tileset {:?}", path.display());
        let resources = get_resource_cache();
        debug!("Got resource cache {:?}", path.display());
        Self::load(path, &resources.tilesets)
    }

    pub fn save_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::save(
            tileset,
            &tileset.resource_path,
            &tileset.name,
            &resources.tilesets,
        )
    }

    pub fn delete_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::delete(&tileset.resource_path, &tileset.name, &resources.tilesets)
    }

    pub fn create_map(map: &GameMap) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::create(map, &map.resource_path, &map.name, &resources.maps)
    }

    pub fn load_map(path: PathBuf) -> Result<GameMap, BlueprintError> {
        let resources = get_resource_cache();
        Self::load(path, &resources.maps)
    }

    pub fn save_map(map: &GameMap) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::save(map, &map.resource_path, &map.name, &resources.maps)
    }

    pub fn delete_map(map: &GameMap) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::delete(&map.resource_path, &map.name, &resources.maps)
    }

    pub fn create_script(map: &Script) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::create(map, &map.resource_path, &map.name, &resources.scripts)
    }

    pub fn load_script(path: PathBuf) -> Result<Script, BlueprintError> {
        let resources = get_resource_cache();
        Self::load(path, &resources.scripts)
    }

    pub fn save_script(map: &Script) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::save(map, &map.resource_path, &map.name, &resources.scripts)
    }

    pub fn delete_script(script: &Script) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::delete(&script.resource_path, &script.name, &resources.scripts)
    }

    pub fn create_character_animation(
        character_animation: &CharacterAnimation,
    ) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::create(
            character_animation,
            &character_animation.resource_path,
            &character_animation.name,
            &resources.character_animations,
        )
    }

    pub fn load_character_animation(path: PathBuf) -> Result<CharacterAnimation, BlueprintError> {
        let resources = get_resource_cache();
        debug!("path {:?}", path.display());
        Self::load(path, &resources.character_animations)
    }

    pub fn save_character_animation(
        character_animation: &CharacterAnimation,
    ) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::save(
            character_animation,
            &character_animation.resource_path,
            &character_animation.name,
            &resources.character_animations,
        )
    }

    pub fn delete_character_animation(
        character_animation: &CharacterAnimation,
    ) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::delete(
            &character_animation.resource_path,
            &character_animation.name,
            &resources.character_animations,
        )
    }

    pub fn create_scene(scene: &Scene) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::create(scene, &scene.resource_path, &scene.name, &resources.scenes)
    }

    pub fn load_scene(path: PathBuf) -> Result<Scene, BlueprintError> {
        let resources = get_resource_cache();
        Self::load(path, &resources.scenes)
    }

    pub fn update_node_in_scene(
        scene: &mut Scene,
        path: Vec<usize>,
        entity_update: EntityUpdateKind,
    ) -> Result<(), BlueprintError> {
        if path.is_empty() {
            scene.root_node.update_with_entity_update(entity_update);
        } else {
            Self::modify_node(&mut scene.root_node, &path[..], |node_to_update| {
                node_to_update.update_with_entity_update(entity_update)
            })?;
        }
        Blueprint::save_scene(scene)
    }

    pub fn add_child_in_scene(
        scene: &mut Scene,
        path: Vec<usize>,
        game_node: GameNodeKind,
    ) -> Result<(), BlueprintError> {
        if path.is_empty() {
            scene.root_node.add_child(game_node);
        } else {
            Self::modify_node_rec(
                &mut scene.root_node,
                &path[..],
                game_node,
                |node_for_insertion, new_child| node_for_insertion.add_child(new_child),
            )?;
        }
        Blueprint::save_scene(scene)
    }

    pub fn remove_child_in_scene(
        scene: &mut Scene,
        path: Vec<usize>,
        game_node: GameNodeKind,
    ) -> Result<(), BlueprintError> {
        if path.is_empty() {
            error!("Removing root node not supported!");
        } else if path.len() == 1 {
            scene.root_node.remove_child(path[0]);
        } else {
            let path_len = path.len();
            let index_to_remove = path[path_len - 1];
            Self::modify_node_rec(
                &mut scene.root_node,
                &path[..path_len - 1],
                game_node,
                |node_for_insertion, _| node_for_insertion.remove_child(index_to_remove),
            )?;
        }
        Blueprint::save_scene(scene)
    }

    fn modify_node<F>(
        current_game_node: &mut GameNodeKind,
        path: &[usize],
        operation: F,
    ) -> Result<(), BlueprintError>
    where
        F: FnOnce(&mut GameNodeKind),
    {
        let children = current_game_node.borrow_children();
        if children.is_empty() || path.is_empty() {
            return Err(BlueprintError::AccessNested(
                "Unable to access node recursively, children empty or path 0!",
            ));
        }
        if path.len() == 1 {
            let child = children.get_mut(path[0]).ok_or_else(|| {
                BlueprintError::AccessNested(
                    "Unable to access node recursively, child not accessible at this position!",
                )
            })?;
            operation(child);
            return Ok(());
        }

        let child = children.get_mut(path[0]).ok_or_else(|| {
            BlueprintError::AccessNested(
                "Unable to access node recursively, child not accessible at this position!",
            )
        })?;
        Self::modify_node(child, &path[1..], operation)
    }

    fn modify_node_rec<F>(
        current_game_node: &mut GameNodeKind,
        path: &[usize],
        game_node_to_insert: GameNodeKind,
        operation: F,
    ) -> Result<(), BlueprintError>
    where
        F: FnOnce(&mut GameNodeKind, GameNodeKind) -> (),
    {
        let children = current_game_node.borrow_children();
        if children.is_empty() || path.is_empty() {
            return Err(BlueprintError::AccessNested(
                "Unable to access node recursively, children empty or path 0!",
            ));
        }
        if path.len() == 1 {
            let child = children.get_mut(path[0]).ok_or_else(|| {
                BlueprintError::AccessNested(
                    "Unable to access node recursively, child not accessible at this position!",
                )
            })?;
            operation(child, game_node_to_insert);
            return Ok(());
        }

        let child = children.get_mut(path[0]).ok_or_else(|| {
            BlueprintError::AccessNested(
                "Unable to access node recursively, child not accessible at this position!",
            )
        })?;
        Self::modify_node_rec(child, &path[1..], game_node_to_insert, operation)
    }

    pub fn save_scene(scene: &Scene) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::save(scene, &scene.resource_path, &scene.name, &resources.scenes)
    }

    pub fn delete_scene(scene: &Scene) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        Self::delete(&scene.resource_path, &scene.name, &resources.scenes)
    }

    pub fn module_exists(module_name: &String) -> bool {
        let resources = get_resource_cache();
        if let Ok(file_path) = Self::path_buf_to_string(
            get_out_dir()
                .join("modules")
                .join(module_name)
                .join(format!("{}.module.json", &module_name)),
        ) {
            if let Ok(modules) = resources
                .modules
                .read()
                .map_err(|_| BlueprintError::ReadPoison("Could not load resource due to poison."))
            {
                return modules.contains_key(&file_path);
            }
        }
        false
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
        remove_dir_all(module_path)?;
        Ok(())
    }

    pub fn path_buf_to_string(path_buf: PathBuf) -> Result<String, BlueprintError> {
        path_buf
            .into_os_string()
            .into_string()
            .map_err(BlueprintError::ConversionToString)
    }

    pub fn create_module(module: &Module) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        let module_path =
            Self::path_buf_to_string(get_out_dir().join("modules").join(&module.name))?;
        Self::create(module, &module_path, &module.name, &resources.modules)
    }

    pub fn lazy_create_module(module_name: &ModuleName) -> Result<Module, BlueprintError> {
        let module = Module::new(module_name.clone(), Uuid::new_v4().to_string());
        let result = Self::create_module(&module);
        if let Err(BlueprintError::FileAlreadyExists) = result {
            Self::load_module(module_name)
        } else {
            Ok(module)
        }
    }

    pub fn load_module(module_name: &String) -> Result<Module, BlueprintError> {
        let resources = get_resource_cache();
        let file_path = get_out_dir().join("modules").join(module_name);
        Self::load(file_path, &resources.modules)
    }

    pub fn save_module(module: &Module) -> Result<(), BlueprintError> {
        let resources = get_resource_cache();
        let file_path_buf = get_out_dir().join("modules").join(&module.name);
        let file_path = file_path_buf
            .to_str()
            .ok_or(BlueprintError::ConversionToStr)?;
        Self::save(module, file_path, &module.name, &resources.modules)
    }

    pub fn get_all_modules() -> Result<Vec<Module>, BlueprintError> {
        let resources = get_resource_cache();
        Ok(resources
            .modules
            .read()
            .map_err(|_| {
                BlueprintError::ReadPoison("Was not able to get all modules due to poison.")
            })?
            .values()
            .cloned()
            .collect())
    }

    pub fn io_points_to_hashset(points: &[IOPoint]) -> HashSet<String> {
        points.iter().map(|p| p.name.clone()).collect()
    }
}
