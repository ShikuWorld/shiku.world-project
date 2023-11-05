use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::core::get_out_dir;
use crate::core::guest::{ModuleEnterSlot, ModuleExitSlot};
use crate::core::module::ModuleName;

pub type EntityId = usize;
pub type JointId = usize;

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Conductor {
    pub(super) module_connection_map: HashMap<ModuleExitSlot, (ModuleName, ModuleEnterSlot)>,
}

impl Conductor {
    pub fn default() -> Conductor {
        Conductor {
            module_connection_map: HashMap::new(),
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum Resource {}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Chunk {
    pub width: u32,
    pub height: u32,
    pub position: (i32, i32),
    pub data: Vec<Vec<u32>>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct IOPoint {
    pub name: String,
    pub condition_script: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Module {
    pub name: String,
    pub resources: Vec<Resource>,
    pub maps: Vec<Map>,
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
    pub resources: Option<Vec<Resource>>,
    pub maps: Option<Vec<Map>>,
    pub insert_points: Option<Vec<IOPoint>>,
    pub exit_points: Option<Vec<IOPoint>>,
    pub max_guests: Option<usize>,
    pub min_guests: Option<usize>,
}

impl Module {
    pub fn new(name: String) -> Module {
        Module {
            name,
            maps: Vec::new(),
            max_guests: 0,
            min_guests: 0,
            exit_points: Vec::new(),
            insert_points: Vec::new(),
            resources: Vec::new(),
            close_after_full: false,
        }
    }
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Map {
    pub name: String,
    pub resources: Vec<Resource>,
    pub entities: Vec<Entity>,
    pub joints: HashMap<JointId, Joint>,
    pub terrain: HashMap<LayerKind, Vec<Chunk>>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Layer {
    pub name: String,
    pub kind: LayerKind,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Entity {
    pub id: EntityId,
    pub physicality: Physicality,
    pub render: Render,
    pub children: Vec<EntityId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Physicality {
    pub position: (Real, Real),
    pub velocity: (Real, Real),
    pub rotation: Real,
    pub body: BodyType,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum BodyType {
    RigidBody(RigidBody),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct RigidBody {
    collision_shape: CollisionShape,
    joints: Vec<JointId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum CollisionShape {}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct Render {
    offset: (Real, Real),
    layer: LayerKind,
    kind: RenderKind,
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
pub enum RenderKind {}

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
    FileDoesNotExist,
    #[error("Tried to write a file that shouldn't already exist")]
    FileAlreadyExists,
    #[error("Could not load blueprint due to IO error.")]
    IOError(#[from] std::io::Error),
    #[error("Could not load blueprint due to malformed json.")]
    SerdeJSONError(#[from] serde_json::error::Error),
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

    pub fn create_module(&self, module_name: String) -> Result<Module, BlueprintError> {
        let dir_path = get_out_dir().join("modules").join(&module_name);

        fs::create_dir_all(&dir_path)?;

        let file_path = dir_path.join(format!("{}.json", &module_name));
        if file_path.exists() {
            return Err(BlueprintError::FileAlreadyExists);
        }

        let module = Module::new(module_name);
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
        let paths = fs::read_dir(dir_path)?;
        let mut modules = Vec::new();
        for path in paths {
            modules.push(
                self.load_module(
                    path?
                        .file_name()
                        .to_os_string()
                        .into_string()
                        .unwrap_or("MODULE_NAME_BROKEN".into()),
                )?,
            )
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
