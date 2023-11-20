use std::fs;
use std::fs::{create_dir_all, remove_dir_all, remove_file, rename, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::str::FromStr;

use log::debug;
use uuid::Uuid;

use crate::core::blueprint::def::{BlueprintError, BlueprintService, Conductor, Module, Tileset};
use crate::core::get_out_dir;

impl Module {
    pub fn new(name: String, id: String) -> Module {
        Module {
            id,
            name,
            maps: Vec::new(),
            max_guests: 0,
            min_guests: 0,
            gid_map: Vec::new(),
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
        fs::create_dir_all(out_dir.join("modules"))?;

        Ok(())
    }

    pub fn module_exists(&self, module_name: &String) -> bool {
        let dir_path = get_out_dir().join("modules").join(module_name);
        let file_path = dir_path.join(format!("{}.json", module_name));
        file_path.exists()
    }

    pub fn change_module_name(
        &self,
        module: &mut Module,
        new_name: String,
    ) -> Result<(), BlueprintError> {
        if self.module_exists(&new_name) {
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

    pub fn create_tileset(&self, tileset: &Tileset) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(tileset.resource_path.as_str())?;
        let directory_path = out_dir.join(resource_path);
        create_dir_all(directory_path.as_path().clone())?;
        let file_path = directory_path.join(format!("{}.tileset.json", tileset.name));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, tileset)?;
        Ok(())
    }

    pub fn save_tileset(&self, tileset: &Tileset) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(tileset.resource_path.as_str())?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!("{}.tileset.json", tileset.name));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist);
        }
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, tileset)?;
        Ok(())
    }

    pub fn delete_tileset(&self, tileset: &Tileset) -> Result<(), BlueprintError> {
        let out_dir = get_out_dir();
        let resource_path = PathBuf::from_str(tileset.resource_path.as_str())?;
        let directory_path = out_dir.join(resource_path);
        let file_path = directory_path.join(format!("{}.tileset.json", tileset.name));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist);
        }
        remove_file(file_path)?;
        Ok(())
    }

    pub fn delete_module(&self, module_name: &String) -> Result<(), BlueprintError> {
        let module_path = get_out_dir().join("modules").join(module_name);
        debug!("Removing {:?}", module_path.to_str());
        remove_dir_all(module_path)?;
        Ok(())
    }

    pub fn create_module(&self, module_name: String) -> Result<Module, BlueprintError> {
        let dir_path = get_out_dir().join("modules").join(&module_name);

        fs::create_dir_all(&dir_path)?;

        let file_path = dir_path.join(format!("{}.json", &module_name));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }

        let module = Module::new(module_name, Uuid::new_v4().to_string());
        self.save_module(&module)?;

        Ok(module)
    }

    pub fn lazy_load_module(&self, module_name: String) -> Result<Module, BlueprintError> {
        let result = self.create_module(module_name.clone());
        if let Err(BlueprintError::FileAlreadyExists) = result {
            self.load_module(module_name)
        } else {
            result
        }
    }

    pub fn get_all_modules(&self) -> Result<Vec<Module>, BlueprintError> {
        let dir_path = get_out_dir().join("modules");
        debug!("1");
        let paths = fs::read_dir(dir_path)?;
        debug!("2");
        let mut modules = Vec::new();
        for path in paths {
            let module_name = path?
                .file_name()
                .to_os_string()
                .into_string()
                .unwrap_or("MODULE_NAME_BROKEN".into());
            modules.push(self.load_module(module_name)?);
            debug!("4");
        }

        Ok(modules)
    }

    pub fn load_module(&self, module_name: String) -> Result<Module, BlueprintError> {
        let dir_path = get_out_dir().join("modules").join(&module_name);
        let file_path = dir_path.join(format!("{}.json", &module_name));
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist);
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save_module(&self, module: &Module) -> Result<(), BlueprintError> {
        let file_path = get_out_dir()
            .join("modules")
            .join(&module.name)
            .join(format!("{}.json", &module.name));
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, module)?)
    }

    pub fn load_conductor_blueprint(&self) -> Result<Conductor, BlueprintError> {
        let file_path = get_out_dir().join("conductor.json");
        if !file_path.exists() {
            return Err(BlueprintError::FileDoesNotExist);
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save_conductor_blueprint(&self, blueprint: &Conductor) -> Result<(), BlueprintError> {
        let file_path = get_out_dir().join("conductor.json");
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, blueprint)?)
    }
}
