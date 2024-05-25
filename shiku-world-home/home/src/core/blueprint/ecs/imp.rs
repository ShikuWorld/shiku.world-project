use std::cell::{BorrowMutError, RefCell, RefMut};
use std::cmp::PartialEq;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

use crate::core::ApiShare;
use log::{debug, error};
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::ColliderHandle;
use rapier2d::math::Vector;
use rhai::{Dynamic, Engine, ImmutableString, Scope, AST};
use smartstring::{SmartString, SmartStringMode};

use crate::core::blueprint::def::ResourcePath;
use crate::core::blueprint::ecs::def::{
    DynamicMap, ECSShared, Entity, EntityMaps, EntityUpdate, EntityUpdateKind, GameNodeScript,
    GameNodeScriptError, GameNodeScriptFunctions, ScopeCacheValue, ECS,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{
    Collider, ColliderKind, ColliderShape, GameNodeKind, GameNodeKindClean, Node2DKind,
    Node2DKindClean, RenderKind, RenderKindClean, RigidBodyType, Scene, SceneId, Script, Transform,
};
use crate::core::guest::ActorId;
use crate::core::rapier_simulation::def::RapierSimulation;

impl From<&Scene> for ECS {
    fn from(scene: &Scene) -> Self {
        let mut new_ecs = ECS::new();
        if let Some(mut shared) = new_ecs.shared.try_borrow_mut() {
            new_ecs.scene_root = Entity(shared.entity_counter);
            new_ecs.scene_name.clone_from(&scene.name);
            new_ecs.scene_resource_path.clone_from(&scene.resource_path);
            new_ecs.scene_id.clone_from(&scene.id);

            let engine = Engine::new();
            Self::add_entity_from_game_node(&scene.root_node, &mut shared, &engine);
        }
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
            entity_scripts: HashMap::new(),
            entities: HashSet::new(),
            shared: ApiShare::new(ECSShared {
                entities: EntityMaps {
                    game_node_id: HashMap::new(),
                    game_node_name: HashMap::new(),
                    game_node_children: HashMap::new(),
                    game_node_kind: HashMap::new(),
                    node_2d_kind: HashMap::new(),
                    node_2d_instance_path: HashMap::new(),
                    node_2d_entity_instance_parent: HashMap::new(),
                    game_node_parent: HashMap::new(),
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
                added_entities: Vec::new(),
                removed_entities: Vec::new(),
                entity_counter: 0,
            }),
        }
    }

    pub fn create_initial_rigid_bodies(&mut self, physics: &mut RapierSimulation) {
        if let Some(mut shared) = self.shared.try_borrow_mut() {
            for (original_entity, rigid_body_type) in shared.entities.rigid_body_type.clone() {
                let possible_instance_entity =
                    { Self::get_instance_entity_if_exists(&original_entity, &shared) };
                let transform = shared
                    .entities
                    .transforms
                    .get(&possible_instance_entity)
                    .cloned()
                    .unwrap_or_else(|| Transform::default());
                Self::add_rigid_body_for_entity(
                    &original_entity,
                    &rigid_body_type,
                    &transform,
                    &mut shared,
                    physics,
                );
            }
        }
    }

    pub fn add_rigid_body_for_entity(
        original_entity: &Entity,
        rigid_body_type: &RigidBodyType,
        transform: &Transform,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        let rigid_body_handle =
            Self::create_rigid_body_from_type(rigid_body_type, transform, physics);
        shared
            .entities
            .rigid_body_handle
            .insert(*original_entity, rigid_body_handle);
    }

    fn get_instance_entity_if_exists(original_entity: &Entity, shared: &ECSShared) -> Entity {
        if let Some(parent_entity) = shared
            .entities
            .node_2d_entity_instance_parent
            .get(original_entity)
        {
            return *parent_entity;
        }

        *original_entity
    }

    fn create_rigid_body_from_type(
        rigid_body_type: &RigidBodyType,
        transform: &Transform,
        physics: &mut RapierSimulation,
    ) -> RigidBodyHandle {
        match rigid_body_type {
            RigidBodyType::Dynamic => {
                physics.add_dynamic_rigid_body(transform.position.0, transform.position.1)
            }
            RigidBodyType::Fixed => {
                physics.add_fixed_rigid_body(transform.position.0, transform.position.1)
            }
            RigidBodyType::KinematicPositionBased => physics
                .add_kinematic_position_based_rigid_body(
                    transform.position.0,
                    transform.position.1,
                ),
            RigidBodyType::KinematicVelocityBased => physics
                .add_kinematic_velocity_based_rigid_body(
                    transform.position.0,
                    transform.position.1,
                ),
        }
    }

    pub fn attach_colliders_to_entity(
        entity: &Entity,
        ecs: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        if let (Some(children), Some(rigid_body_handle)) = (
            ecs.entities.game_node_children.get(entity),
            ecs.entities.rigid_body_handle.get(entity),
        ) {
            for child_entity in children {
                if let Some(child_collider) = ecs.entities.collider.get(child_entity) {
                    let child_collider_handle =
                        Self::create_collider(child_collider, rigid_body_handle, physics);
                    ecs.entities
                        .collider_handle
                        .insert(*child_entity, child_collider_handle);
                    debug!("Successfully attached collider 1");
                }
            }
        }
    }

    pub fn attach_collider_to_its_entity(
        parent_entity: &Entity,
        child_entity: &Entity,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        if let (Some(child_collider), Some(parent_rigid_body_handle)) = (
            shared.entities.collider.get(child_entity),
            shared.entities.rigid_body_handle.get(parent_entity),
        ) {
            let child_collider_handle =
                Self::create_collider(child_collider, parent_rigid_body_handle, physics);
            shared
                .entities
                .collider_handle
                .insert(*child_entity, child_collider_handle);
            debug!("Successfully attached collider 2");
        }
    }

    pub fn attach_initial_colliders_to_rigid_bodies(&mut self, physics: &mut RapierSimulation) {
        if let Some(mut shared) = self.shared.try_borrow_mut() {
            Self::_attach_initial_colliders_to_rigid_bodies(&mut shared, physics);
        }
    }

    fn _attach_initial_colliders_to_rigid_bodies(
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        for (parent_entity, children) in &shared.entities.game_node_children {
            if let Some(rigid_body_handle) = shared.entities.rigid_body_handle.get(parent_entity) {
                for child_entity in children {
                    if let Some(child_collider) = shared.entities.collider.get(child_entity) {
                        let child_collider_handle =
                            Self::create_collider(child_collider, rigid_body_handle, physics);
                        shared
                            .entities
                            .collider_handle
                            .insert(*child_entity, child_collider_handle);
                        debug!("Successfully attached collider 2");
                    }
                }
            }
        }
    }

    fn create_collider(
        collider: &Collider,
        rigid_body_handle: &RigidBodyHandle,
        physics: &mut RapierSimulation,
    ) -> ColliderHandle {
        let is_sensor = match collider.kind {
            ColliderKind::Solid => false,
            ColliderKind::Sensor => true,
        };
        match collider.shape {
            ColliderShape::Ball(radius) => {
                physics.create_ball_collider(radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::CapsuleX(half_y, radius) => {
                physics.create_capsule_x_collider(half_y, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::CapsuleY(half_x, radius) => {
                physics.create_capsule_y_collider(half_x, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::Cuboid(half_x, half_y) => {
                physics.create_cuboid_collider(half_x, half_y, *rigid_body_handle, is_sensor)
            }
        }
    }

    pub fn add_child_to_entity(
        parent_entity: Entity,
        child: &GameNodeKind,
        shared: &mut ECSShared,
        engine: &Engine,
    ) -> Entity {
        let child_entity = Entity(shared.entity_counter);
        shared
            .entities
            .game_node_children
            .entry(parent_entity)
            .or_default()
            .push(child_entity);
        shared
            .entities
            .game_node_parent
            .insert(child_entity, parent_entity);
        if let Some(Node2DKindClean::Instance) = shared.entities.node_2d_kind.get(&parent_entity) {
            shared
                .entities
                .node_2d_entity_instance_parent
                .insert(child_entity, parent_entity);
        }
        Self::add_entity_from_game_node(child, shared, engine)
    }

    fn add_entity_from_game_node(
        node_kind: &GameNodeKind,
        ecs: &mut ECSShared,
        engine: &Engine,
    ) -> Entity {
        let entity = Entity(ecs.entity_counter);
        let mut script_path = None;
        ecs.entity_counter += 1;
        match node_kind {
            GameNodeKind::Node2D(node_2d) => {
                ecs.entities
                    .game_node_kind
                    .insert(entity, GameNodeKindClean::Node2D);
                ecs.entities.game_node_id.insert(entity, node_2d.id.clone());
                script_path = node_2d.script.clone();
                ecs.entities.game_node_children.insert(entity, Vec::new());
                ecs.entities
                    .game_node_name
                    .insert(entity, node_2d.name.clone());
                ecs.entities
                    .transforms
                    .insert(entity, node_2d.data.transform.clone());

                match &node_2d.data.kind {
                    Node2DKind::Instance(resource_path) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Instance);
                        ecs.entities
                            .node_2d_instance_path
                            .insert(entity, resource_path.clone());
                    }
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
        }
        ecs.added_entities.push((entity.clone(), script_path));
        if let Some(instance_root_node) = Self::get_node_2d_instance_root_node(node_kind) {
            debug!("# Adding instance root node");
            Self::add_child_to_entity(entity, &instance_root_node, ecs, engine);
        } else {
            for child in node_kind.get_children() {
                Self::add_child_to_entity(entity, child, ecs, engine);
            }
        }

        entity
    }

    fn get_node_2d_instance_root_node(game_node_kind: &GameNodeKind) -> Option<GameNodeKind> {
        match game_node_kind {
            GameNodeKind::Node2D(node_2d) => match &node_2d.data.kind {
                Node2DKind::Instance(resource_path) => {
                    match Blueprint::load_scene(resource_path.clone().into()) {
                        Ok(scene) => Some(scene.root_node),
                        Err(e) => {
                            error!("Error loading scene: {:?}", e);
                            None
                        }
                    }
                }
                _ => None,
            },
        }
    }

    pub fn remove_script_on_all_entities(&mut self, resource_path: &ResourcePath) {
        self.entity_scripts
            .retain(|_, script| script.path != *resource_path);
    }

    pub fn update(&mut self) {
        if let Some(mut shared) = self.shared.try_borrow_mut() {
            for (new_entity, resource_path) in shared
                .added_entities
                .drain(..)
                .filter_map(|(e, p)| p.map(|r| (e, r)))
            {
                self.entity_scripts.insert(
                    new_entity,
                    GameNodeScript::new(new_entity, &Engine::new(), resource_path).unwrap(),
                );
            }
            for (new_entity) in shared.removed_entities.drain(..) {
                self.entity_scripts.remove(&new_entity);
            }
        }
    }

    pub fn apply_entity_update_s(
        entity_scripts: &mut HashMap<Entity, GameNodeScript>,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
        entity_update: EntityUpdate,
        engine: &Engine,
    ) {
        let entity = entity_update.id;

        match entity_update.kind {
            EntityUpdateKind::InstancePath(_) => {
                error!("Instances should not exist on themselves inside ECS for now!");
            }
            EntityUpdateKind::Collider(collider) => {
                shared.entities.collider.insert(entity, collider.clone());
                if let Some(collider_handle) = shared.entities.collider_handle.get(&entity).cloned()
                {
                    if let Some(parent) =
                        shared
                            .entities
                            .game_node_parent
                            .get(&entity)
                            .and_then(|parent| {
                                if shared.entities.rigid_body_handle.contains_key(parent) {
                                    Some(*parent)
                                } else {
                                    None
                                }
                            })
                    {
                        physics.remove_collider(collider_handle);
                        shared.entities.collider_handle.remove(&entity);
                        ECS::attach_collider_to_its_entity(&parent, &entity, shared, physics);
                    }
                }
            }
            EntityUpdateKind::Transform(transform) => {
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(transform.position.0, transform.position.1),
                        transform.rotation,
                        *rigid_body_handle,
                    );
                } else {
                    shared.entities.transforms.insert(entity, transform);
                }
            }
            EntityUpdateKind::PositionRotation((x, y, r)) => {
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(x, y),
                        r,
                        *rigid_body_handle,
                    );
                } else if let Some(transform) = shared.entities.transforms.get_mut(&entity) {
                    transform.position = (x, y);
                    transform.rotation = r;
                }
            }
            EntityUpdateKind::Name(name) => {
                shared.entities.game_node_name.insert(entity, name);
            }
            EntityUpdateKind::ScriptPath(script_path_option) => match script_path_option {
                Some(script_path) => match GameNodeScript::new(entity, engine, script_path) {
                    Ok(game_node_script) => {
                        entity_scripts.insert(entity, game_node_script);
                    }
                    Err(e) => {
                        error!("Error creating script in apply entity update: {:?}", e);
                    }
                },
                None => {
                    entity_scripts.remove(&entity);
                }
            },
            EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                shared
                    .entities
                    .rigid_body_type
                    .insert(entity, rigid_body_type.clone());
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    physics.remove_rigid_body(*rigid_body_handle);
                    shared.entities.rigid_body_handle.remove(&entity);
                    let transform = shared
                        .entities
                        .transforms
                        .get(&entity)
                        .cloned()
                        .unwrap_or_default();
                    ECS::add_rigid_body_for_entity(
                        &entity,
                        &rigid_body_type,
                        &transform,
                        shared,
                        physics,
                    );
                    ECS::attach_colliders_to_entity(&entity, shared, physics);
                }
            }
            EntityUpdateKind::Gid(gid) => {
                shared.entities.render_gid.insert(entity, gid);
            }
            EntityUpdateKind::UpdateScriptScope(scope_key, scope_value) => {
                if let Some(game_node_script) = entity_scripts.get_mut(&entity) {
                    game_node_script.update_scope(scope_key, scope_value);
                }
            }
            EntityUpdateKind::SetScriptScope(scope_cache) => {
                if let Some(game_node_script) = entity_scripts.get_mut(&entity) {
                    for (key, value) in scope_cache {
                        game_node_script.update_scope(key, value);
                    }
                }
            }
        }
    }
}

impl ECSShared {
    pub fn get_instance_root_entity(&self, entity: &Entity) -> Option<&Entity> {
        if let Some(Node2DKindClean::Instance) = self.entities.node_2d_kind.get(entity) {
            return self
                .entities
                .game_node_children
                .get(entity)
                .and_then(|children| children.first());
        }
        None
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
        let game_node_script_functions = Self::get_game_node_script_functions_from_ast(&ast);
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
                        if (*scope_value - *cache_value).abs() > 0.0001_f64 {
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
                            debug!("Updating map");
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
        self.game_node_script_functions = Self::get_game_node_script_functions_from_ast(&self.ast);
        self.update_scope_from_script(engine);
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

    pub fn call_actor_joined(&mut self, engine: &Engine, actor_id: &ActorId) {
        if self.game_node_script_functions.actor_joined {
            match engine.call_fn::<()>(&mut self.scope, &self.ast, "actor_joined", (*actor_id,)) {
                Ok(()) => {}
                Err(e) => error!("Error calling actor_joined function: {:?}", e),
            }
        }
    }

    pub fn call_actor_left(&mut self, engine: &Engine, actor_id: &ActorId) {
        if self.game_node_script_functions.actor_left {
            match engine.call_fn::<()>(&mut self.scope, &self.ast, "actor_left", (*actor_id,)) {
                Ok(()) => {}
                Err(e) => error!("Error calling actor_left function: {:?}", e),
            }
        }
    }

    fn get_game_node_script_functions_from_ast(ast: &AST) -> GameNodeScriptFunctions {
        let mut functions = GameNodeScriptFunctions {
            init: false,
            update: false,
            actor_joined: false,
            actor_left: false,
        };
        for fun in ast.iter_functions() {
            match fun.name {
                "init" => functions.init = true,
                "update" => functions.update = true,
                "actor_joined" => functions.actor_joined = true,
                "actor_left" => functions.actor_left = true,
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

impl PartialEq for ScopeCacheValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ScopeCacheValue::String(a), ScopeCacheValue::String(b)) => a == b,
            (ScopeCacheValue::Number(a), ScopeCacheValue::Number(b)) => (a - b).abs() < 0.0001_f64,
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
