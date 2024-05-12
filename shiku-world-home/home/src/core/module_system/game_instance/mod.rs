use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use log::{debug, error};
use rapier2d::prelude::Real;
use rhai::{Engine, ParseError, AST};
use snowflake::SnowflakeIdBucket;
use thiserror::Error;

use crate::core::blueprint::def::{
    BlueprintError, BlueprintResource, Chunk, GameMap, Gid, LayerKind, ResourceKind, TerrainParams,
};
use crate::core::blueprint::def::{Module, ResourcePath};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{CollisionShape, Script};
use crate::core::guest::{ActorId, Admin, Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication, AdminEnterSuccessState, AdminLeftSuccessState, EnterFailedState,
    EnterSuccessState, LeaveFailedState, LeaveSuccessState, ModuleInputReceiver, ModuleInputSender,
    ModuleOutputReceiver, ModuleOutputSender, ModuleToSystemEvent, SystemToModuleEvent,
};
use crate::core::module_system::def::DynamicGameModule;
use crate::core::module_system::error::{CreateWorldError, DestroyWorldError};
use crate::core::module_system::world::WorldId;
use crate::core::{send_and_log_error, TARGET_FRAME_DURATION};
use crate::resource_module::def::{LoadResource, ResourceModule};
use crate::resource_module::errors::ResourceParseError;

#[derive(Error, Debug)]
pub enum CreateInstanceManagerError {
    #[error(transparent)]
    BlueprintError(#[from] BlueprintError),
    #[error(transparent)]
    ResourceParse(#[from] ResourceParseError),
}

pub type GameInstanceId = String;

pub struct GameInstanceManager {
    pub(crate) game_instances: HashMap<GameInstanceId, GameInstance>,
    pub(crate) inactive_game_instances: Vec<GameInstanceId>,
    pub(crate) connected_actor_ids: HashSet<ActorId>,
    pub(crate) guest_to_game_instance_map: HashMap<ActorId, GameInstanceId>,
    pub(crate) active_admins: HashMap<ActorId, HashSet<GameInstanceId>>,
    pub(crate) input_receiver: ModuleInputReceiver,
    pub(crate) output_sender: ModuleOutputSender,
    pub(crate) module_blueprint: Module,
    pub(crate) script_ast_cache: AstCache,
    pub(crate) game_instance_timeout: Real,
    pub(crate) instance_id_gen: SnowflakeIdBucket,
}

#[derive(Debug)]
pub struct AstCache {
    pub init: HashMap<ResourcePath, AST>,
    pub update: HashMap<ResourcePath, AST>,
}

impl AstCache {
    pub fn new() -> AstCache {
        AstCache {
            update: HashMap::new(),
            init: HashMap::new(),
        }
    }

    pub fn compile_and_cache_script(
        &mut self,
        engine: &Engine,
        script: &Script,
    ) -> Result<(), ParseError> {
        engine.compile(&script.content).map(|ast| {
            let script_resource_path =
                format!("{:?}/{:?}.script.json", script.resource_path, script.name);

            self.init.remove(&script_resource_path);
            self.update.remove(&script_resource_path);

            for def in ast.iter_functions() {
                match def.name {
                    "init" => {
                        self.init.insert(script_resource_path.clone(), ast.clone());
                    }
                    "update" => {
                        self.update
                            .insert(script_resource_path.clone(), ast.clone());
                    }
                    _ => {}
                }
            }
        })
    }

    pub fn remove_script(&mut self, resource_path: &ResourcePath) {
        self.init.remove(resource_path);
        self.update.remove(resource_path);
    }
}

impl GameInstanceManager {
    pub fn new(
        module_blueprint: Module,
        resource_module: &mut ResourceModule,
    ) -> Result<
        (GameInstanceManager, ModuleInputSender, ModuleOutputReceiver),
        CreateInstanceManagerError,
    > {
        let (input_sender, input_receiver, output_sender, output_receiver) =
            create_module_communication();
        let mut manager = GameInstanceManager {
            game_instances: HashMap::new(),
            inactive_game_instances: Vec::new(),
            guest_to_game_instance_map: HashMap::new(),
            active_admins: HashMap::new(),
            connected_actor_ids: HashSet::new(),
            instance_id_gen: SnowflakeIdBucket::new(1, 6),
            game_instance_timeout: 30000.0,
            script_ast_cache: AstCache::new(),
            input_receiver,
            output_sender,
            module_blueprint,
        };

        manager.register_resources(resource_module);
        manager.init_script_ast_cache();

        Ok((manager, input_sender, output_receiver))
    }

    pub fn update_gid_collision_shape_map(
        &mut self,
        gid: &Gid,
        collision_shape: &Option<CollisionShape>,
    ) {
        for game_instance in self.game_instances.values_mut() {
            game_instance
                .dynamic_module
                .update_gid_collision_shape_map(gid, collision_shape);
        }
    }

    pub fn update(&mut self) {
        self.relay_messages_to_correct_instances();

        for game_instance in self.game_instances.values_mut() {
            game_instance.update(&self.module_blueprint, &self.script_ast_cache);
            if !game_instance.dynamic_module.guests.is_empty()
                || !game_instance.dynamic_module.admins.is_empty()
            {
                game_instance.inactive_time = 0.0;
            }
            game_instance.inactive_time += TARGET_FRAME_DURATION;
            if game_instance.inactive_time > self.game_instance_timeout {
                self.inactive_game_instances.push(game_instance.id.clone());
                debug!("Closing game instance.");
            }
        }

        for inactive_game_instanced_id in self.inactive_game_instances.drain(..) {
            self.game_instances.remove(&inactive_game_instanced_id);
            send_and_log_error(
                &mut self.output_sender.module_to_system_sender,
                ModuleToSystemEvent::GameInstanceClosed(
                    self.module_blueprint.id.clone(),
                    inactive_game_instanced_id,
                ),
            );
        }
    }

    pub fn update_world_map(&mut self, world_id: &WorldId, layer_kind: &LayerKind, chunk: &Chunk) {
        for game_instance in self.game_instances.values_mut() {
            game_instance
                .dynamic_module
                .update_world_map(world_id, layer_kind, chunk);
        }
    }

    pub fn get_active_actor_ids(&self) -> Vec<ActorId> {
        let mut active_actors = Vec::new();
        active_actors.extend(
            self.guest_to_game_instance_map
                .iter()
                .filter(|(actor_id, game_instance_id)| {
                    if let Some(game_instance) = self.game_instances.get(*game_instance_id) {
                        if let Some(guest) = game_instance.dynamic_module.guests.get(*actor_id) {
                            return guest.guest_com.connected;
                        }
                    }
                    false
                })
                .map(|(actor_id, _)| actor_id.clone()),
        );
        active_actors.extend(self.active_admins.keys());

        active_actors
    }

    pub fn let_admin_into_instance(
        &mut self,
        admin: &Admin,
        instance_id: GameInstanceId,
        world_id: WorldId,
    ) -> Result<AdminEnterSuccessState, EnterFailedState> {
        let admin_active_instances = self.active_admins.entry(admin.id).or_default();
        let mut success_state = AdminEnterSuccessState::EnteredWorld;
        if !admin_active_instances.contains(&instance_id) {
            admin_active_instances.insert(instance_id.clone());
            success_state = AdminEnterSuccessState::EnteredInstanceAndWorld;
        }
        if let Some(instance) = self.game_instances.get_mut(&instance_id) {
            instance.dynamic_module.let_admin_enter(admin, world_id)?;
        }
        self.connected_actor_ids.insert(admin.id);
        Ok(success_state)
    }

    pub fn let_admin_leave_instance(
        &mut self,
        admin: &Admin,
        instance_id: GameInstanceId,
        world_id: WorldId,
    ) -> Result<AdminLeftSuccessState, LeaveFailedState> {
        let mut success_state = AdminLeftSuccessState::LeftWorld;
        let active_admin_instances = self.active_admins.entry(admin.id).or_default();
        if !active_admin_instances.contains(&instance_id) {
            return Err(LeaveFailedState::NotInModule);
        }
        if let Some(instance) = self.game_instances.get_mut(&instance_id) {
            instance.dynamic_module.let_admin_leave(admin, world_id)?;
            if !instance.dynamic_module.admins.contains_key(&admin.id) {
                success_state = AdminLeftSuccessState::LeftWorldAndInstance;
            }
        }
        active_admin_instances.remove(&instance_id);
        self.connected_actor_ids.remove(&admin.id);
        Ok(success_state)
    }

    pub fn try_enter(
        &mut self,
        guest: &Guest,
        module_enter_slot: &ModuleEnterSlot,
    ) -> Result<(GameInstanceId, EnterSuccessState), EnterFailedState> {
        if self.guest_to_game_instance_map.contains_key(&guest.id) {
            return Err(EnterFailedState::AlreadyEntered);
        }

        let game_instance_id = self.lazy_get_game_instance_for_guest_to_join();
        if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
            return match game_instance
                .dynamic_module
                .try_enter(guest, module_enter_slot)
            {
                Ok(success_state) => {
                    let game_instance_id = game_instance.id.clone();
                    self.guest_to_game_instance_map
                        .insert(guest.id, game_instance_id.clone());
                    self.connected_actor_ids.insert(guest.id);
                    if self.module_blueprint.close_after_full
                        && game_instance.dynamic_module.guests.len()
                            >= self.module_blueprint.max_guests
                    {
                        game_instance.closed = true;
                    }
                    Ok((game_instance_id, success_state))
                }
                Err(fail_state) => Err(fail_state),
            };
        }

        Err(EnterFailedState::GameInstanceNotFoundWTF)
    }

    pub fn try_leave(
        &mut self,
        guest: &Guest,
    ) -> Result<(GameInstanceId, LeaveSuccessState), LeaveFailedState> {
        if let Some(game_instance_id) = self.guest_to_game_instance_map.remove(&guest.id) {
            if let Some(game_instance) = self.game_instances.get_mut(&game_instance_id) {
                return match game_instance.dynamic_module.try_leave(guest) {
                    Ok(success_state) => {
                        self.connected_actor_ids.remove(&guest.id);
                        Ok((game_instance_id, success_state))
                    }
                    Err(err) => Err(err),
                };
            }
        }

        Err(LeaveFailedState::NotInModule)
    }

    pub fn create_world(
        &mut self,
        game_map: &GameMap,
    ) -> HashMap<GameInstanceId, Result<WorldId, CreateWorldError>> {
        self.game_instances
            .values_mut()
            .map(|v| {
                (
                    v.id.clone(),
                    v.dynamic_module
                        .create_world(game_map, &self.script_ast_cache),
                )
            })
            .collect()
    }

    pub fn destroy_world(
        &mut self,
        game_map: &GameMap,
    ) -> HashMap<GameInstanceId, Result<WorldId, DestroyWorldError>> {
        self.game_instances
            .values_mut()
            .map(|v| (v.id.clone(), v.dynamic_module.destroy_world(game_map)))
            .collect()
    }

    fn loading_resources_from_blueprint_resource(
        blueprint_resource: &BlueprintResource,
    ) -> Vec<LoadResource> {
        match blueprint_resource.kind {
            ResourceKind::Tileset => {
                match Blueprint::load_tileset(PathBuf::from(blueprint_resource.path.clone())) {
                    Ok(tileset) => tileset
                        .get_image_paths()
                        .iter()
                        .map(|path| LoadResource::image(path.clone()))
                        .collect(),
                    Err(err) => {
                        error!("Could not load tileset! {:?}", err);
                        Vec::new()
                    }
                }
            }
            ResourceKind::Scene
            | ResourceKind::Map
            | ResourceKind::Unknown
            | ResourceKind::Script => Vec::new(),
        }
    }

    pub fn register_resources(&self, resource_module: &mut ResourceModule) {
        resource_module.init_resources_for_module(self.module_blueprint.id.clone());
        for resource in &self.module_blueprint.resources {
            Self::loading_resources_from_blueprint_resource(resource)
                .into_iter()
                .for_each(|resource| {
                    debug!("Registering {:?}", resource);
                    resource_module
                        .register_resource_for_module(self.module_blueprint.id.clone(), resource)
                });
        }
    }

    pub fn update_script_ast_cache(&mut self, updated_resources: &[BlueprintResource]) {
        let engine = Engine::new();
        let current_script_set: HashSet<String> = self
            .module_blueprint
            .resources
            .iter()
            .filter_map(|r| {
                if r.kind == ResourceKind::Script {
                    Some(r.path.clone())
                } else {
                    None
                }
            })
            .collect();
        let updated_script_set: HashSet<String> = updated_resources
            .iter()
            .filter_map(|r| {
                if r.kind == ResourceKind::Script {
                    Some(r.path.clone())
                } else {
                    None
                }
            })
            .collect();
        for insertion in updated_script_set.difference(&current_script_set) {
            match Blueprint::load_script(insertion.into()) {
                Ok(script) => {
                    match self
                        .script_ast_cache
                        .compile_and_cache_script(&engine, &script)
                    {
                        Ok(()) => {}
                        Err(err) => {
                            error!("Was not able to compile newly add new script {err}")
                        }
                    }
                }
                Err(err) => {
                    error!("Was not able to load new script resource {err}")
                }
            }
        }
        for deletion in current_script_set.difference(&updated_script_set) {
            self.script_ast_cache.remove_script(deletion);
        }
    }

    fn init_script_ast_cache(&mut self) {
        let engine = Engine::new();
        for resource in &self.module_blueprint.resources {
            if ResourceKind::Script == resource.kind {
                match Blueprint::load_script(resource.path.clone().into()) {
                    Ok(script) => match self
                        .script_ast_cache
                        .compile_and_cache_script(&engine, &script)
                    {
                        Ok(()) => {}
                        Err(err) => error!("Could not compile script: {err}"),
                    },
                    Err(err) => error!("Could not load script: {err}"),
                }
            }
        }
    }

    fn relay_messages_to_correct_instances(&mut self) {
        for message in self.input_receiver.guest_to_module_receiver.drain() {
            if let Some(game_instance) =
                self.game_instances.get_mut(&message.event_type.instance_id)
            {
                if let Err(err) = game_instance
                    .input_sender
                    .guest_to_module_sender
                    .send(message)
                {
                    error!(
                        "Game instance message could not send guest message to module?! {:?}",
                        err
                    );
                }
            }
        }

        for message in self.input_receiver.system_to_module_receiver.drain() {
            if let Some(game_instance) = self.game_instances.get_mut(&message.instance_id) {
                match message.event_type {
                    SystemToModuleEvent::Disconnected(actor_id) => {
                        game_instance.dynamic_module.actor_disconnected(&actor_id);
                        self.connected_actor_ids.remove(&actor_id);
                    }
                    SystemToModuleEvent::Reconnected(actor_id) => {
                        game_instance.dynamic_module.actor_reconnected(&actor_id);
                        self.connected_actor_ids.insert(actor_id);
                    }
                }
            }
        }
    }

    pub fn create_new_game_instance(&mut self) -> GameInstanceId {
        let new_game_instance = GameInstance::new(
            self.instance_id_gen.get_id().to_string(),
            &self.module_blueprint,
            &self.script_ast_cache,
            self.output_sender.clone(),
        );
        let new_game_instance_id = new_game_instance.id.clone();
        self.game_instances
            .entry(new_game_instance.id.clone())
            .or_insert(new_game_instance);
        send_and_log_error(
            &mut self.output_sender.module_to_system_sender,
            ModuleToSystemEvent::GameInstanceCreated(
                self.module_blueprint.id.clone(),
                new_game_instance_id.clone(),
            ),
        );

        new_game_instance_id
    }

    pub fn lazy_get_game_instance_for_guest_to_join(&mut self) -> GameInstanceId {
        let max_guest_count = self.module_blueprint.max_guests;
        let mut game_instance_id_found = None;

        for game_instance in self.game_instances.values_mut() {
            if !game_instance.closed && game_instance.dynamic_module.guests.len() < max_guest_count
            {
                game_instance_id_found = Some(game_instance.id.clone());
                break;
            }
        }

        if let Some(game_instance_id) = game_instance_id_found {
            return game_instance_id;
        }

        self.create_new_game_instance()
    }

    pub fn get_terrain_params_for_guest(
        &self,
        guest_id: &ActorId,
        game_instance_id: &GameInstanceId,
    ) -> Option<TerrainParams> {
        if let Some(instance) = self.game_instances.get(game_instance_id) {
            if let Some(world_id) = instance.dynamic_module.guest_to_world.get(guest_id) {
                return instance.dynamic_module.get_terrain_params(world_id);
            }
        }
        None
    }

    pub fn get_terrain_params_for_admin(
        &self,
        _guest_id: &ActorId,
        game_instance_id: &GameInstanceId,
        world_id: &WorldId,
    ) -> Option<TerrainParams> {
        if let Some(instance) = self.game_instances.get(game_instance_id) {
            return instance.dynamic_module.get_terrain_params(world_id);
        }
        None
    }
}

pub struct GameInstance {
    pub(crate) id: GameInstanceId,
    pub(crate) inactive_time: Real,
    pub(crate) dynamic_module: DynamicGameModule,
    pub(crate) input_sender: ModuleInputSender,
    pub(crate) closed: bool,
}

impl GameInstance {
    pub fn new(
        id: GameInstanceId,
        module: &Module,
        script_ast_cache: &AstCache,
        output_sender: ModuleOutputSender,
    ) -> GameInstance {
        let (dynamic_module, input_sender) =
            DynamicGameModule::create(id.clone(), module, script_ast_cache, output_sender);
        GameInstance {
            id,
            dynamic_module,
            input_sender,
            inactive_time: 0.0,
            closed: false,
        }
    }

    pub fn update(&mut self, module: &Module, script_ast_cache: &AstCache) {
        self.dynamic_module.update(module, script_ast_cache);
    }
}
