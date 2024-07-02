use crate::core::blueprint::def::{BlueprintError, ResourcePath};
use crate::core::blueprint::ecs::def::{DynamicMap, Entity};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::Script;
use crate::core::guest::ActorId;
use log::{debug, error};
use rhai::{Dynamic, Engine, FuncArgs, ImmutableString, ParseError, Scope, AST};
use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};
use std::collections::{BTreeMap, HashMap};
use ts_rs::TS;

#[derive(Debug)]
pub struct GameNodeScript {
    pub path: ResourcePath,
    pub ast: AST,
    pub entity: Entity,
    pub scope_cache: HashMap<String, ScopeCacheValue>,
    pub scope: Scope<'static>,
    pub(crate) game_node_script_functions: HashMap<GameNodeScriptFunction, &'static str>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum GameNodeScriptFunction {
    Init,
    Update,
    InstanceReset,
    ActorJoined,
    ActorLeft,
    ScriptReload,
}

impl GameNodeScriptFunction {
    pub fn map_from_ast(ast: &AST) -> HashMap<GameNodeScriptFunction, &'static str> {
        let mut hash_map = HashMap::new();

        for fun in ast.iter_functions() {
            match fun.name {
                "init" => {
                    hash_map.insert(GameNodeScriptFunction::Init, "init");
                }
                "update" => {
                    hash_map.insert(GameNodeScriptFunction::Update, "update");
                }
                "instance_reset" => {
                    hash_map.insert(GameNodeScriptFunction::InstanceReset, "instance_reset");
                }
                "actor_joined" => {
                    hash_map.insert(GameNodeScriptFunction::ActorJoined, "actor_joined");
                }
                "actor_left" => {
                    hash_map.insert(GameNodeScriptFunction::ActorLeft, "actor_left");
                }
                "script_reload" => {
                    hash_map.insert(GameNodeScriptFunction::ScriptReload, "script_reload");
                }
                _ => {}
            }
        }
        hash_map
    }
}

#[derive(Debug)]
pub enum GameNodeScriptError {
    BlueprintError(BlueprintError),
    CompileError(ParseError),
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub enum ScopeCacheValue {
    String(String),
    Number(f64),
    Integer(i64),
    Map(HashMap<String, ScopeCacheValue>),
}

pub const MIN_EQUAL_FLOAT_VALUE: f64 = 0.0000001_f64;

impl ScopeCacheValue {
    pub(crate) fn equals_dynamic_value(
        scope_cache_value: &ScopeCacheValue,
        dynamic_value: &Dynamic,
    ) -> bool {
        match scope_cache_value {
            ScopeCacheValue::String(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<String>() {
                    *value == *dynamic_value
                } else {
                    false
                }
            }
            ScopeCacheValue::Number(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<f64>() {
                    (value - *dynamic_value).abs() < MIN_EQUAL_FLOAT_VALUE
                } else {
                    false
                }
            }
            ScopeCacheValue::Integer(value) => {
                if let Some(dynamic_value) = dynamic_value.read_lock::<i64>() {
                    *value == *dynamic_value
                } else {
                    false
                }
            }
            ScopeCacheValue::Map(scope_cache_map) => {
                if let Some(dynamic_map) = dynamic_value.read_lock::<DynamicMap>() {
                    scope_cache_map.iter().all(|(key, cache_val)| {
                        let smart_string: SmartString<LazyCompact> = key.into();
                        match dynamic_map.get(&smart_string) {
                            Some(dyn_val) => {
                                ScopeCacheValue::equals_dynamic_value(cache_val, dyn_val)
                            }
                            None => false,
                        }
                    })
                } else {
                    false
                }
            }
        }
    }
}

impl GameNodeScript {
    pub fn new(
        entity: Entity,
        engine: &Engine,
        path: ResourcePath,
    ) -> Result<Self, GameNodeScriptError> {
        Self::compile(engine, path.clone()).map(|ast| {
            let mut game_node_script = Self::from_ast(entity, path, ast);
            game_node_script.update_scope_from_script(engine);
            game_node_script
        })
    }

    pub fn update_scope(&mut self, scope_key: String, scope_value: ScopeCacheValue) {
        let dynamic_value: Dynamic = scope_value.into();
        debug!(
            "Updating scope key: {} with value: {:?}",
            scope_key, dynamic_value
        );
        self.scope.set_value(&scope_key, dynamic_value);
    }

    pub fn from_ast(entity: Entity, path: ResourcePath, ast: AST) -> Self {
        let game_node_script_functions = GameNodeScriptFunction::map_from_ast(&ast);
        let scope = Scope::new();
        Self {
            entity,
            path,
            ast,
            scope_cache: HashMap::new(),
            scope,
            game_node_script_functions,
        }
    }

    pub fn init_scope_cache_from_scope(
        scope_cache: &mut HashMap<String, ScopeCacheValue>,
        scope: &Scope,
    ) {
        for (key, _, value) in scope.iter() {
            scope_cache.insert(key.into(), value.clone().into());
        }
    }

    pub fn update_scope_cache(&mut self) -> Option<HashMap<String, ScopeCacheValue>> {
        let mut updated = false;

        for (key, value) in self.scope_cache.iter_mut() {
            match value {
                ScopeCacheValue::String(cache_value) => {
                    if let Some(scope_value) = self
                        .scope
                        .get(key)
                        .and_then(|v| v.read_lock::<ImmutableString>())
                    {
                        if *scope_value != *cache_value {
                            *value = ScopeCacheValue::String(scope_value.clone().into());
                            updated = true;
                        }
                    }
                }
                ScopeCacheValue::Number(cache_value) => {
                    if let Some(scope_value) =
                        self.scope.get(key).and_then(|v| v.read_lock::<f64>())
                    {
                        if (*scope_value - *cache_value).abs() > MIN_EQUAL_FLOAT_VALUE {
                            *value = ScopeCacheValue::Number(*scope_value);
                            updated = true;
                        }
                    }
                }
                ScopeCacheValue::Integer(cache_value) => {
                    if let Some(scope_value) =
                        self.scope.get(key).and_then(|v| v.read_lock::<i64>())
                    {
                        if *scope_value != *cache_value {
                            *value = ScopeCacheValue::Integer(*scope_value);
                            updated = true;
                        }
                    }
                }
                ScopeCacheValue::Map(cache_value) => {
                    if let Some(scope_value) = self
                        .scope
                        .get(key)
                        .and_then(|v| v.read_lock::<DynamicMap>())
                    {
                        for (key, value) in scope_value.iter() {
                            match cache_value.get(key.as_str()) {
                                Some(cache_value) => {
                                    if !ScopeCacheValue::equals_dynamic_value(cache_value, value) {
                                        updated = true;
                                    }
                                }
                                None => {
                                    updated = true;
                                }
                            }
                        }
                        if updated {
                            let mut new_map = HashMap::new();
                            for (key, value) in scope_value.iter() {
                                new_map.insert(key.clone().into(), value.clone().into());
                            }
                            *value = ScopeCacheValue::Map(new_map);
                        }
                    }
                }
            }
        }
        if updated {
            Some(self.scope_cache.clone())
        } else {
            None
        }
    }

    pub fn update_scope_from_script(&mut self, engine: &Engine) {
        let mut new_scope = Scope::new();
        match engine.run_ast_with_scope(&mut new_scope, &self.ast) {
            Ok(()) => {
                let mut new_scope_cache = HashMap::new();
                Self::init_scope_cache_from_scope(&mut new_scope_cache, &new_scope);
                let scope_cache = &mut self.scope_cache;
                // Remove keys not present in new scope
                scope_cache.retain(|k, _| new_scope_cache.contains_key(k));
                // Add new values to scope cache but keep old values
                for (key, value) in new_scope_cache {
                    scope_cache.entry(key).or_insert(value);
                }
                self.scope.clear();
                for (key, value) in scope_cache.iter() {
                    let dynamic_value: Dynamic = value.clone().into();
                    self.scope.set_value(key.clone(), dynamic_value);
                }
                self.scope
                    .push_constant("ENTITY_ID", Dynamic::from(self.entity));
                debug!("Scope update successful");
            }
            Err(e) => error!("Error updating scope: {:?}", e),
        }
    }

    pub fn reset_from_new_ast(&mut self, engine: &Engine, ast: AST) {
        self.ast = ast;
        self.game_node_script_functions = GameNodeScriptFunction::map_from_ast(&self.ast);
        self.update_scope_from_script(engine);
    }

    pub fn call(
        &mut self,
        script_fun: GameNodeScriptFunction,
        engine: &Engine,
        args: impl FuncArgs + Sized,
    ) {
        if let Some(name) = self.game_node_script_functions.get(&script_fun) {
            match engine.call_fn::<()>(&mut self.scope, &self.ast, name, args) {
                Ok(()) => {}
                Err(e) => error!("Error calling {name} function: {:?}", e),
            }
        }
    }

    fn compile(engine: &Engine, path: ResourcePath) -> Result<AST, GameNodeScriptError> {
        match Blueprint::load_script(path.clone().into()) {
            Ok(script) => Self::compile_from_script(engine, &script),
            Err(e) => Err(GameNodeScriptError::BlueprintError(e)),
        }
    }

    fn compile_from_script(engine: &Engine, script: &Script) -> Result<AST, GameNodeScriptError> {
        match engine.compile(&script.content) {
            Ok(ast) => Ok(ast),
            Err(e) => Err(GameNodeScriptError::CompileError(e)),
        }
    }
}

impl PartialEq for ScopeCacheValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ScopeCacheValue::String(a), ScopeCacheValue::String(b)) => a == b,
            (ScopeCacheValue::Number(a), ScopeCacheValue::Number(b)) => {
                (a - b).abs() < MIN_EQUAL_FLOAT_VALUE
            }
            (ScopeCacheValue::Integer(a), ScopeCacheValue::Integer(b)) => a == b,
            (ScopeCacheValue::Map(a), ScopeCacheValue::Map(b)) => a == b,
            _ => false,
        }
    }
}

impl Into<Dynamic> for ScopeCacheValue {
    fn into(self) -> Dynamic {
        match self {
            ScopeCacheValue::String(val) => Dynamic::from(val),
            ScopeCacheValue::Number(val) => Dynamic::from(val),
            ScopeCacheValue::Integer(val) => Dynamic::from(val),
            ScopeCacheValue::Map(map) => {
                let mut dynamic_map: DynamicMap = BTreeMap::new();
                for (key, value) in map {
                    dynamic_map.insert(SmartString::from(key), value.into());
                }
                Dynamic::from(dynamic_map)
            }
        }
    }
}

impl Into<ScopeCacheValue> for Dynamic {
    fn into(self) -> ScopeCacheValue {
        match self.type_name() {
            "string" => ScopeCacheValue::String(self.try_cast::<String>().unwrap_or_else(|| {
                error!("Error casting Dynamic to String");
                String::default()
            })),
            "f64" => ScopeCacheValue::Number(self.try_cast::<f64>().unwrap_or_else(|| {
                error!("Error casting Dynamic to f64");
                0.0
            })),
            "i64" => ScopeCacheValue::Integer(self.try_cast::<i64>().unwrap_or_else(|| {
                error!("Error casting Dynamic to i64");
                0
            })),
            "map" => {
                let mut map = HashMap::new();
                let cast_value = self.try_cast::<DynamicMap>().unwrap_or_else(|| {
                    error!("Error casting Dynamic to HashMap<ImmutableString, Dynamic>");
                    BTreeMap::new()
                });
                for (key, value) in cast_value {
                    map.insert(key.into(), value.into());
                }
                ScopeCacheValue::Map(map)
            }
            type_name => ScopeCacheValue::String(format!("Unknown type: {type_name}")),
        }
    }
}
