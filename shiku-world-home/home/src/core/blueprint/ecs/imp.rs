use std::collections::HashMap;
use std::ops::Deref;

use log::{debug, error};
use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};

use crate::core::blueprint::def::{BlueprintError, ResourcePath};
use crate::core::blueprint::ecs::def::{
    Entity, EntityMaps, EntityUpdate, EntityUpdateKind, GameNodeScript, GameNodeScriptError,
    GameNodeScriptFunctions, ScopeCacheValue, ECS,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{
    GameNodeKind, GameNodeKindClean, Node2DKind, Node2DKindClean, RenderKind, RenderKindClean,
    Scene, SceneId, Script,
};

impl From<&Scene> for ECS {
    fn from(scene: &Scene) -> Self {
        let mut new_ecs = ECS::new();
        new_ecs.scene_root = Entity(new_ecs.entity_counter);
        new_ecs.scene_name.clone_from(&scene.name);
        new_ecs.scene_resource_path.clone_from(&scene.resource_path);
        new_ecs.scene_id.clone_from(&scene.id);

        let engine = Engine::new();
        Self::add_entity(&scene.root_node, &mut new_ecs, &engine);
        new_ecs
    }
}

impl ECS {
    pub fn new() -> ECS {
        ECS {
            scene_root: Entity::default(),
            scene_name: String::default(),
            scene_resource_path: ResourcePath::default(),
            scene_id: SceneId::default(),
            entity_counter: 0,
            entities: EntityMaps {
                game_node_script: HashMap::new(),
                game_node_id: HashMap::new(),
                game_node_name: HashMap::new(),
                game_node_children: HashMap::new(),
                game_node_kind: HashMap::new(),
                node_2d_kind: HashMap::new(),
                render_kind: HashMap::new(),
                render_offset: HashMap::new(),
                render_layer: HashMap::new(),
                render_gid: HashMap::new(),
                transforms: HashMap::new(),
                rigid_body_velocity: HashMap::new(),
                rigid_body_type: HashMap::new(),
                rigid_body_handle: HashMap::new(),
                collider: HashMap::new(),
                collider_handle: HashMap::new(),
                dirty: HashMap::new(),
            },
        }
    }

    pub fn add_child_to_entity(
        parent_entity: Entity,
        child: &GameNodeKind,
        ecs: &mut ECS,
        engine: &Engine,
    ) -> Entity {
        ecs.entities
            .game_node_children
            .entry(parent_entity)
            .or_default()
            .push(Entity(ecs.entity_counter));
        Self::add_entity(child, ecs, engine)
    }

    fn add_entity(node_kind: &GameNodeKind, ecs: &mut ECS, engine: &Engine) -> Entity {
        let entity = Entity(ecs.entity_counter);
        ecs.entity_counter += 1;

        match node_kind {
            GameNodeKind::Node2D(node_2d) => {
                ecs.entities
                    .game_node_kind
                    .insert(entity, GameNodeKindClean::Node2D);
                ecs.entities.game_node_id.insert(entity, node_2d.id.clone());
                if let Some(resource_path) = &node_2d.script {
                    match GameNodeScript::new(engine, resource_path.clone()) {
                        Ok(mut game_node_script) => {
                            game_node_script.init_scope(engine);
                            ecs.entities
                                .game_node_script
                                .insert(entity, game_node_script);
                        }
                        Err(e) => {
                            error!("Error creating script in apply entity update: {:?}", e);
                        }
                    }
                }
                ecs.entities.game_node_children.insert(entity, Vec::new());
                ecs.entities
                    .game_node_name
                    .insert(entity, node_2d.name.clone());
                ecs.entities
                    .transforms
                    .insert(entity, node_2d.data.transform.clone());

                match &node_2d.data.kind {
                    Node2DKind::Node2D(_) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Node2D);
                    }
                    Node2DKind::RigidBody(rigid_body) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::RigidBody);
                        ecs.entities
                            .rigid_body_type
                            .insert(entity, rigid_body.body.clone());
                        ecs.entities
                            .rigid_body_velocity
                            .insert(entity, rigid_body.velocity);
                    }
                    Node2DKind::Collider(collider) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Collider);
                        ecs.entities.collider.insert(entity, collider.clone());
                    }
                    Node2DKind::Render(render) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Render);
                        ecs.entities
                            .render_layer
                            .insert(entity, render.layer.clone());
                        ecs.entities.render_offset.insert(entity, render.offset);
                        match render.kind {
                            RenderKind::AnimatedSprite(gid) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::AnimatedSprite);
                                ecs.entities.render_gid.insert(entity, gid);
                            }
                            RenderKind::Sprite(gid) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::Sprite);
                                ecs.entities.render_gid.insert(entity, gid);
                            }
                        }
                    }
                }
            }
            GameNodeKind::Instance(_node) => {
                error!("Instance not implemented!");
            }
        }
        for child in node_kind.get_children() {
            Self::add_child_to_entity(entity, child, ecs, engine);
        }

        entity
    }

    pub fn remove_script_on_all_entities(&mut self, resource_path: &ResourcePath) {
        self.entities
            .game_node_script
            .retain(|_, script| script.path != *resource_path);
    }

    pub fn apply_entity_update(&mut self, entity_update: EntityUpdate, engine: &Engine) {
        let entity = entity_update.id;
        match entity_update.kind {
            EntityUpdateKind::Transform(transform) => {
                self.entities.transforms.insert(entity, transform);
            }
            EntityUpdateKind::Name(name) => {
                self.entities.game_node_name.insert(entity, name);
            }
            EntityUpdateKind::ScriptPath(script_path_option) => match script_path_option {
                Some(script_path) => match GameNodeScript::new(engine, script_path) {
                    Ok(game_node_script) => {
                        self.entities
                            .game_node_script
                            .insert(entity, game_node_script);
                    }
                    Err(e) => {
                        error!("Error creating script in apply entity update: {:?}", e);
                    }
                },
                None => {
                    self.entities.game_node_script.remove(&entity);
                }
            },
            EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                self.entities
                    .rigid_body_type
                    .insert(entity, rigid_body_type);
            }
            EntityUpdateKind::PositionRotation((x, y, r)) => {
                if let Some(transform) = self.entities.transforms.get_mut(&entity) {
                    transform.position = (x, y);
                    transform.rotation = r;
                }
            }
            EntityUpdateKind::Gid(gid) => {
                self.entities.render_gid.insert(entity, gid);
            }
            EntityUpdateKind::UpdateScriptScope(scope_key, scope_value) => {
                if let Some(game_node_script) = self.entities.game_node_script.get_mut(&entity) {
                    game_node_script.update_scope(scope_key, scope_value);
                }
            }
            EntityUpdateKind::SetScriptScope(scope_cache) => {
                if let Some(game_node_script) = self.entities.game_node_script.get_mut(&entity) {
                    for (key, value) in scope_cache {
                        game_node_script.update_scope(key, value);
                    }
                }
            }
        }
    }
}

impl GameNodeScript {
    pub fn new(engine: &Engine, path: ResourcePath) -> Result<Self, GameNodeScriptError> {
        Self::compile(engine, path.clone()).map(|ast| {
            let mut game_node_script = Self::from_ast(path, ast);
            game_node_script.init_scope(engine);
            game_node_script
        })
    }

    pub fn update_scope(&mut self, scope_key: String, scope_value: ScopeCacheValue) {
        match scope_value {
            ScopeCacheValue::String(value) => {
                debug!("Setting string value: {} {:?}", scope_key, value);
                self.scope.set_value(&scope_key, value);
            }
            ScopeCacheValue::Number(value) => {
                debug!("Setting number value: {} {:?}", scope_key, value);
                self.scope.set_value(&scope_key, value);
            }
        }
    }

    pub fn from_ast(path: ResourcePath, ast: AST) -> Self {
        let game_node_script_functions = Self::get_game_node_script_functions_from_ast(&ast);
        let scope = Scope::new();
        Self {
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
            if let Some(value) = value.read_lock::<ImmutableString>() {
                debug!("Inserting into scope cache: {} {:?}", key, value);
                scope_cache.insert(key.into(), ScopeCacheValue::String(value.clone().into()));
            } else if let Some(value) = value.read_lock::<f64>() {
                debug!("Inserting into scope cache: {} {:?}", key, value);
                scope_cache.insert(key.into(), ScopeCacheValue::Number(*value));
            }
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
                            debug!("Comparing {} {:?} {:?}", key, scope_value, cache_value);
                            *value = ScopeCacheValue::String(scope_value.clone().into());
                            updated = true;
                        }
                    }
                }
                ScopeCacheValue::Number(cache_value) => {
                    if let Some(scope_value) =
                        self.scope.get(key).and_then(|v| v.read_lock::<f64>())
                    {
                        if (*scope_value - *cache_value).abs() > 0.0001_f64 {
                            debug!("Comparing {} {:?} {:?}", key, scope_value, cache_value);
                            *value = ScopeCacheValue::Number(*scope_value);
                            updated = true;
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

    pub fn init_scope(&mut self, engine: &Engine) {
        self.scope.clear();
        match engine.run_ast_with_scope(&mut self.scope, &self.ast) {
            Ok(()) => {
                debug!("Scope initialized successfully");
                Self::init_scope_cache_from_scope(&mut self.scope_cache, &self.scope);
            }
            Err(e) => error!("Error initializing scope: {:?}", e),
        }
    }

    pub fn reset_from_new_ast(&mut self, engine: &Engine, ast: AST) {
        self.ast = ast;
        self.game_node_script_functions = Self::get_game_node_script_functions_from_ast(&self.ast);
        self.init_scope(engine);
    }

    pub fn call_init(&mut self, engine: &Engine) {
        if self.game_node_script_functions.init {
            match engine.call_fn::<()>(&mut self.scope, &self.ast, "init", ()) {
                Ok(()) => {}
                Err(e) => error!("Error calling init function: {:?}", e),
            }
        }
    }

    pub fn call_update(&mut self, engine: &Engine) {
        if self.game_node_script_functions.update {
            match engine.call_fn::<()>(&mut self.scope, &self.ast, "update", ()) {
                Ok(()) => {}
                Err(e) => error!("Error calling update function: {:?}", e),
            }
        }
    }

    fn get_game_node_script_functions_from_ast(ast: &AST) -> GameNodeScriptFunctions {
        let mut functions = GameNodeScriptFunctions {
            init: false,
            update: false,
        };
        for fun in ast.iter_functions() {
            match fun.name {
                "init" => functions.init = true,
                "update" => functions.update = true,
                _ => {}
            }
        }
        functions
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
