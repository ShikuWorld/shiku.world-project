use crate::core::blueprint::def::{BlueprintError, GameMap, IOPoint, Tileset};
use crate::core::get_out_dir;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{create_dir_all, remove_file, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;

pub struct Blueprint;

impl Blueprint {
    pub fn create<T: Serialize>(
        resource: &T,
        resource_path: &String,
        resource_name: &String,
        file_extension: &str,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path.as_str())?;
        let directory_path = out_dir.join(resource_path);
        create_dir_all(directory_path.as_path())?;
        let file_path = directory_path.join(format!("{}.{}.json", resource_name, file_extension));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, resource)?;
        Ok(())
    }

    pub fn load<T: DeserializeOwned>(path: PathBuf) -> Result<T, BlueprintError> {
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

    pub fn save<T: Serialize>(
        resource: &T,
        resource_path: &str,
        resource_name: &str,
        file_extension: &str,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path)?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!("{}.{}.json", resource_name, file_extension));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!("{:?}", file_path)));
        }
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, resource)?;
        Ok(())
    }

    pub fn delete(
        resource_path: &String,
        resource_name: &String,
        file_extension: &str,
    ) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(resource_path.as_str())?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!("{}.{}.json", resource_name, file_extension));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist(format!("{:?}", file_path)));
        }
        remove_file(file_path)?;
        Ok(())
    }

    pub fn create_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        Self::create(tileset, &tileset.resource_path, &tileset.name, "tileset")
    }

    pub fn load_tileset(path: PathBuf) -> Result<Tileset, BlueprintError> {
        Self::load(path)
    }

    pub fn save_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        Self::save(tileset, &tileset.resource_path, &tileset.name, "tileset")
    }

    pub fn delete_tileset(tileset: &Tileset) -> Result<(), BlueprintError> {
        Self::delete(&tileset.resource_path, &tileset.name, "tileset")
    }

    pub fn create_map(map: &GameMap) -> Result<(), BlueprintError> {
        Self::create(map, &map.resource_path, &map.name, "map")
    }

    pub fn load_map(path: PathBuf) -> Result<GameMap, BlueprintError> {
        Self::load(path)
    }

    pub fn save_map(map: &GameMap) -> Result<(), BlueprintError> {
        Self::save(map, &map.resource_path, &map.name, "map")
    }

    pub fn delete_map(map: &GameMap) -> Result<(), BlueprintError> {
        Self::delete(&map.resource_path, &map.name, "map")
    }

    pub fn create_scene(map: &GameMap) -> Result<(), BlueprintError> {
        Self::create(map, &map.resource_path, &map.name, "scene")
    }

    pub fn load_map(path: PathBuf) -> Result<GameMap, BlueprintError> {
        Self::load(path)
    }

    pub fn save_map(map: &GameMap) -> Result<(), BlueprintError> {
        Self::save(map, &map.resource_path, &map.name, "map")
    }

    pub fn delete_map(map: &GameMap) -> Result<(), BlueprintError> {
        Self::delete(&map.resource_path, &map.name, "map")
    }

    pub fn io_points_to_hashset(points: &Vec<IOPoint>) -> HashSet<String> {
        points.clone().into_iter().map(|p| p.name).collect()
    }
}
