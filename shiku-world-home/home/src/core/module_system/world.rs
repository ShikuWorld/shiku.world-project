use std::collections::{HashMap, HashSet};

use log::{debug, error};
use rand::{thread_rng, Rng};
use rapier2d::na::Dyn;
use rapier2d::prelude::*;
use rhai::{exported_module, Dynamic, Engine, FuncRegistration, Module as RhaiModule};

use crate::core::blueprint::character_animation::{CharacterDirection, StateId};
use crate::core::blueprint::def::{GameMap, Gid, JsonResource, ResourcePath, TerrainParams};
use crate::core::blueprint::ecs::def::{ECSShared, Entity, EntityMaps, EntityUpdate, ECS};
use crate::core::blueprint::ecs::game_node_script::GameNodeScriptFunction;
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{CollisionShape, GameNodeKind, Transform};
use crate::core::guest::ActorId;
use crate::core::module::{GameSystemToGuestEvent, GuestInput};
use crate::core::module_system::error::CreateWorldError;
use crate::core::module_system::script_types::CharacterDirectionModule;
use crate::core::module_system::terrain_manager::TerrainManager;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::{ApiShare, TARGET_FRAME_DURATION};

pub type WorldId = String;

const MIN_EQUAL_FLOAT_VALUE: f32 = 0.00001;
pub type ParentEntity = Entity;
pub type ChildEntity = Entity;
pub struct Events {
    pub add_entity_events: Vec<(ParentEntity, ChildEntity)>,
}

pub struct World {
    pub world_id: WorldId,
    pub game_map_path: ResourcePath,
    pub physics: ApiShare<RapierSimulation>,
    pub actor_api: ApiShare<ActorApi>,
    pub event_cache: ApiShare<Events>,
    pub terrain_manager: TerrainManager,
    pub ecs: ECS,
    pub script_engine: Engine,
}

pub struct ActorApi {
    active_users: HashSet<ActorId>,
    actor_inputs: HashMap<ActorId, GuestInput>,
}

impl ActorApi {
    pub fn get_actor_input(&self, actor_id: &ActorId) -> Option<&GuestInput> {
        self.actor_inputs.get(actor_id)
    }

    pub fn set_actor_input(&mut self, actor_id: ActorId, guest_input: GuestInput) {
        self.actor_inputs.insert(actor_id, guest_input);
    }
}

impl World {
    pub fn new(
        game_map: &GameMap,
        collision_shape_map: &HashMap<Gid, CollisionShape>,
    ) -> Result<World, CreateWorldError> {
        let world_scene = Blueprint::load_scene(game_map.main_scene.clone().into())?;
        let mut physics = RapierSimulation::new();
        let terrain_manager = TerrainManager::new(
            TerrainParams {
                chunk_size: game_map.chunk_size,
                tile_height: game_map.tile_height,
                tile_width: game_map.tile_width,
            },
            game_map.terrain.clone(),
            game_map.layer_parallax.clone(),
            collision_shape_map,
            &mut physics,
        );

        let mut world = World {
            world_id: game_map.world_id.clone(),
            physics: ApiShare::new(physics),
            game_map_path: game_map.get_full_resource_path(),
            event_cache: ApiShare::new(Events {
                add_entity_events: Vec::new(),
            }),
            actor_api: ApiShare::new(ActorApi {
                actor_inputs: HashMap::new(),
                active_users: HashSet::new(),
            }),
            terrain_manager,
            ecs: ECS::from(&world_scene),
            script_engine: Engine::new(),
        };

        world.reset()?;

        Ok(world)
    }

    pub fn reset(&mut self) -> Result<(), CreateWorldError> {
        let game_map = Blueprint::load_map(self.game_map_path.clone().into())?;
        let world_scene = Blueprint::load_scene(game_map.main_scene.clone().into())?;

        let mut ecs = ECS::from(&world_scene);
        let mut physics = RapierSimulation::new();
        Self::init_physics_simulation_from_ecs(&mut ecs, &mut physics);
        self.terrain_manager.re_add_polylines(&mut physics);
        let physics_share = ApiShare::new(physics);
        let mut script_engine = Engine::new();
        Self::register_types(&mut script_engine);
        Self::setup_nodes_api(
            &mut script_engine,
            &mut ecs,
            &physics_share,
            &self.event_cache,
        );
        Self::setup_utils_scripting_api(&mut script_engine);
        Self::setup_physics_scripting_api(&mut script_engine, &physics_share, &mut ecs);
        Self::setup_animation_api(&mut script_engine, &mut ecs);
        Self::setup_actor_api(&mut script_engine, &self.actor_api);
        ecs.process_added_and_removed_entities_and_scope_sets(&script_engine);
        self.ecs = ecs;
        self.physics = physics_share;
        self.script_engine = script_engine;

        Ok(())
    }

    pub fn update(&mut self) {
        if let Some(mut physics) = self.physics.try_borrow_mut() {
            physics.update();
            if let Some(mut shared_ecs) = self.ecs.shared.try_borrow_mut() {
                Self::update_entities_gid_from_animations(&mut shared_ecs);
                Self::update_kinematic_character_controllers(&shared_ecs, &mut physics);
                Self::update_positions(&mut physics, &mut shared_ecs);
            }
        }
        self.ecs
            .process_added_and_removed_entities_and_scope_sets(&self.script_engine);
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call(GameNodeScriptFunction::Update, &self.script_engine, ());
        }
    }

    pub fn update_entities_gid_from_animations(shared: &mut ECSShared) {
        for (entity, character_animation) in shared.entities.character_animation.iter_mut() {
            character_animation.run_current_animation(TARGET_FRAME_DURATION);
            if let Some(current_gid) = shared.entities.render_gid.get_mut(entity) {
                if character_animation.current_gid != *current_gid {
                    *current_gid = character_animation.current_gid;
                    shared.entities.view_dirty.insert(*entity, true);
                }
            }
        }
    }

    pub fn update_kinematic_character_controllers(
        ecs_shared: &ECSShared,
        physics: &mut RapierSimulation,
    ) {
        for (entity, kinematic_character_controller) in
            ecs_shared.entities.kinematic_character.iter()
        {
            if let (Some(children), Some(rigid_body_handle)) = (
                ecs_shared.entities.game_node_children.get(entity),
                ecs_shared.entities.rigid_body_handle.get(entity),
            ) {
                let mut child_collider_handle = None;
                for child in children {
                    if let Some(collider_handle) = ecs_shared.entities.collider_handle.get(child) {
                        child_collider_handle = Some(*collider_handle);
                        break;
                    }
                }
                if let Some(collider_handle) = child_collider_handle {
                    physics.move_character(
                        &kinematic_character_controller.controller,
                        *rigid_body_handle,
                        collider_handle,
                        kinematic_character_controller.desired_translation,
                    );
                }
            }
        }
    }

    pub fn actor_joined_world(&mut self, actor_id: ActorId) {
        if let Some(mut actor_api) = self.actor_api.try_borrow_mut() {
            actor_api.active_users.insert(actor_id);
        }
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call(
                GameNodeScriptFunction::ActorJoined,
                &self.script_engine,
                (actor_id,),
            );
        }
    }

    pub fn actor_left_world(&mut self, actor_id: ActorId) {
        if let Some(mut actor_api) = self.actor_api.try_borrow_mut() {
            actor_api.active_users.remove(&actor_id);
        }
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call(
                GameNodeScriptFunction::ActorLeft,
                &self.script_engine,
                (actor_id,),
            );
        }
    }

    pub fn actor_disconnected(&mut self, actor_id: ActorId) {
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call(
                GameNodeScriptFunction::ActorDisconnected,
                &self.script_engine,
                (actor_id,),
            );
        }
    }

    pub fn actor_reconnected(&mut self, actor_id: ActorId) {
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call(
                GameNodeScriptFunction::ActorReconnected,
                &self.script_engine,
                (actor_id,),
            );
        }
    }

    fn setup_utils_scripting_api(engine: &mut Engine) {
        let mut module = RhaiModule::new();
        let add_fixed_rigid_body = move |start: f64, length: f64| -> Dynamic {
            let mut rng = thread_rng();
            let random_num: f64 = rng.gen();
            Dynamic::from(start + length * random_num)
        };
        FuncRegistration::new("random_num_in_range")
            .set_into_module(&mut module, add_fixed_rigid_body);

        engine.register_static_module("shiku::utils", module.into());
    }

    fn setup_physics_scripting_api(
        engine: &mut Engine,
        physics_share: &ApiShare<RapierSimulation>,
        ecs: &mut ECS,
    ) {
        let mut module = RhaiModule::new();
        let physics_clone = physics_share.clone();
        let add_fixed_rigid_body = move |x: Real, y: Real| {
            if let Some(mut physics) = physics_clone.try_borrow_mut() {
                physics.add_fixed_rigid_body(x, y);
            }
        };
        FuncRegistration::new("add_fixed_rigid_body")
            .set_into_module(&mut module, add_fixed_rigid_body);

        let ecs_shared = ecs.shared.clone();
        let get_rigid_body_handle = move |entity: Entity| -> Dynamic {
            if let Some(shared) = ecs_shared.try_borrow_mut() {
                if let Some(rigid_body_entity) = shared.entities.rigid_body_handle.get(&entity) {
                    return Dynamic::from(*rigid_body_entity);
                }
            }
            Dynamic::from(())
        };
        FuncRegistration::new("get_rigid_body_handle")
            .set_into_module(&mut module, get_rigid_body_handle);

        let ecs_shared = ecs.shared.clone();
        let set_entity_desired_translation = move |entity: Entity, x: f64, y: f64| {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                if let Some(character) = shared.entities.kinematic_character.get_mut(&entity) {
                    character.desired_translation.x = x as f32;
                    character.desired_translation.y = y as f32;
                } else {
                    error!("Could not find kinematic character for entity: {}", entity);
                }
            }
        };
        FuncRegistration::new("set_entity_desired_translation")
            .set_into_module(&mut module, set_entity_desired_translation);

        let physics_clone = physics_share.clone();
        let ecs_shared = ecs.shared.clone();
        let add_force_to_rigid_body = move |entity: Entity, force_x: f64, force_y: f64| {
            if let (Some(mut physics), Some(shared)) =
                (physics_clone.try_borrow_mut(), ecs_shared.try_borrow_mut())
            {
                if let Some(rigid_body_handle) = shared.entities.rigid_body_handle.get(&entity) {
                    physics.s_apply_force(
                        *rigid_body_handle,
                        Vector::new(force_x as Real, force_y as Real),
                    );
                }
            }
        };
        FuncRegistration::new("add_force_to_rigid_body")
            .set_into_module(&mut module, add_force_to_rigid_body);

        let physics_clone = physics_share.clone();
        let ecs_shared = ecs.shared.clone();
        let apply_impulse_to_rigid_body = move |entity: Entity, force_x: f64, force_y: f64| {
            if let (Some(mut physics), Some(shared)) =
                (physics_clone.try_borrow_mut(), ecs_shared.try_borrow_mut())
            {
                if let Some(rigid_body_handle) = shared.entities.rigid_body_handle.get(&entity) {
                    physics.apply_impulse(
                        *rigid_body_handle,
                        Vector::new(force_x as Real, force_y as Real),
                    );
                }
            }
        };
        FuncRegistration::new("apply_impulse_to_rigid_body")
            .set_into_module(&mut module, apply_impulse_to_rigid_body);

        engine.register_static_module("shiku::physics", module.into());
    }

    fn register_types(engine: &mut Engine) {
        engine.register_static_module(
            "CharacterDirection",
            exported_module!(CharacterDirectionModule).into(),
        );
    }

    fn setup_nodes_api(
        engine: &mut Engine,
        ecs: &mut ECS,
        physics_share: &ApiShare<RapierSimulation>,
        event_cache: &ApiShare<Events>,
    ) {
        let mut module = RhaiModule::new();

        let ecs_shared = ecs.shared.clone();
        let get_child_animation_entity = move |entity: Entity| -> Dynamic {
            if let Some(shared) = ecs_shared.try_borrow_mut() {
                if let Some(children) = shared.entities.game_node_children.get(&entity) {
                    for child in children {
                        if shared.entities.character_animation.contains_key(child) {
                            return Dynamic::from(*child);
                        }
                    }
                }
            }
            Dynamic::from(())
        };
        FuncRegistration::new("get_child_animation_entity")
            .set_into_module(&mut module, get_child_animation_entity);

        let ecs_shared = ecs.shared.clone();
        let physics_clone = physics_share.clone();
        let event_cache_clone = event_cache.clone();
        let spawn_entity_from_scene =
            move |parent_entity: Entity, source: &str, x: f64, y: f64| -> Dynamic {
                if let (Some(mut physics), Some(mut shared), Some(mut events)) = (
                    physics_clone.try_borrow_mut(),
                    ecs_shared.try_borrow_mut(),
                    event_cache_clone.try_borrow_mut(),
                ) {
                    match Blueprint::load_scene(source.into()) {
                        Ok(mut scene) => {
                            let new_child_id = Self::_add_entity(
                                &mut shared,
                                &mut physics,
                                parent_entity,
                                &mut scene.root_node,
                                (x as Real, y as Real),
                            );

                            events.add_entity_events.push((parent_entity, new_child_id));

                            return Dynamic::from(new_child_id);
                        }
                        Err(err) => {
                            eprintln!("Error loading scene when spawning entity in api: {:?}", err);
                        }
                    }
                }
                Dynamic::from(())
            };
        FuncRegistration::new("spawn_entity_from_scene")
            .set_into_module(&mut module, spawn_entity_from_scene);

        let ecs_shared = ecs.shared.clone();
        let set_entity_scope_variable = move |entity: Entity, key: &str, value: Dynamic| {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                shared
                    .set_scope_variables
                    .entry(entity)
                    .or_default()
                    .insert(key.to_string(), value.into());
            }
        };
        FuncRegistration::new("set_scope_variable_on_entity")
            .set_into_module(&mut module, set_entity_scope_variable);

        engine.register_static_module("shiku::nodes", module.into());
    }

    fn setup_animation_api(engine: &mut Engine, ecs: &mut ECS) {
        let mut module = RhaiModule::new();

        let ecs_shared = ecs.shared.clone();
        let get_state = move |entity: Entity| -> Dynamic {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                if let Some(animation) = shared.entities.character_animation.get_mut(&entity) {
                    return Dynamic::from(animation.current_state);
                }
            }
            Dynamic::from(())
        };
        FuncRegistration::new("get_state").set_into_module(&mut module, get_state);

        let ecs_shared = ecs.shared.clone();
        let go_to_state = move |entity: Entity, state_id: i64| {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                if let Some(animation) = shared.entities.character_animation.get_mut(&entity) {
                    animation.go_to_state(state_id as StateId);
                }
            }
        };
        FuncRegistration::new("go_to_state").set_into_module(&mut module, go_to_state);

        let ecs_shared = ecs.shared.clone();
        let get_progress = move |entity: Entity| -> f32 {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                if let Some(animation) = shared.entities.character_animation.get_mut(&entity) {
                    return animation.get_animation_progress();
                }
            }
            0.0
        };
        FuncRegistration::new("get_progress").set_into_module(&mut module, get_progress);

        let ecs_shared = ecs.shared.clone();
        let set_direction = move |entity: Entity, direction: CharacterDirection| {
            if let Some(mut shared) = ecs_shared.try_borrow_mut() {
                if let Some(animation) = shared.entities.character_animation.get_mut(&entity) {
                    animation.current_direction = direction;
                }
            }
        };
        FuncRegistration::new("set_direction").set_into_module(&mut module, set_direction);

        engine.register_static_module("shiku::animation", module.into());
    }

    fn setup_actor_api(engine: &mut Engine, actor_api_share: &ApiShare<ActorApi>) {
        let mut module = RhaiModule::new();
        let actor_api_share_clone = actor_api_share.clone();
        FuncRegistration::new("is_key_down").set_into_module(
            &mut module,
            move |actor_id: ActorId, key: &str| {
                if let Some(actor_api) = actor_api_share_clone.try_borrow_mut() {
                    if let Some(guest_input) = actor_api.get_actor_input(&actor_id) {
                        return match key {
                            "right" => guest_input.right,
                            "left" => guest_input.left,
                            "up" => guest_input.up,
                            "down" => guest_input.down,
                            "start" => guest_input.start,
                            "action_1" => guest_input.action_1,
                            "action_2" => guest_input.action_2,
                            _ => false,
                        };
                    }
                }
                false
            },
        );
        let actor_api_share_clone = actor_api_share.clone();
        FuncRegistration::new("get_active_actors").set_into_module(
            &mut module,
            move || -> Vec<Dynamic> {
                actor_api_share_clone
                    .try_borrow_mut()
                    .map(|actor_api| {
                        actor_api
                            .active_users
                            .iter()
                            .cloned()
                            .map(Dynamic::from)
                            .collect::<Vec<Dynamic>>()
                    })
                    .unwrap_or_default()
            },
        );
        engine.register_static_module("shiku::actors", module.into());
    }

    fn update_positions(physics: &mut RapierSimulation, shared: &mut ECSShared) {
        for (entity, rigid_body_handle) in shared.entities.rigid_body_handle.iter() {
            if let Some(transform) = shared.entities.transforms.get_mut(entity) {
                let (x, y, r) = physics.get_rigid_body_translation(*rigid_body_handle);
                if transform.position.0 != x
                    || transform.position.1 != y
                    || (transform.rotation - r).abs() > MIN_EQUAL_FLOAT_VALUE
                {
                    transform.position = (x, y);
                    transform.rotation = r;
                    shared.entities.dirty.insert(*entity, true);
                }
            }
        }
    }

    fn init_physics_simulation_from_ecs(ecs: &mut ECS, physics: &mut RapierSimulation) {
        ecs.create_initial_rigid_bodies(physics);
        ecs.attach_initial_colliders_to_rigid_bodies(physics);
    }

    pub fn apply_admin_entity_update(&mut self, entity_update: EntityUpdate) {
        if let (Some(mut shared), Some(mut physics)) = (
            self.ecs.shared.try_borrow_mut(),
            self.physics.try_borrow_mut(),
        ) {
            ECS::apply_entity_update_s(
                &mut self.ecs.entity_scripts,
                &mut shared,
                &mut physics,
                entity_update,
                &self.script_engine,
            );
        }
    }

    fn _add_entity(
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
        parent_entity: Entity,
        child: &mut GameNodeKind,
        start_position: (Real, Real),
    ) -> Entity {
        if let GameNodeKind::Node2D(node_2d) = child {
            node_2d.data.transform.position = start_position;
        }
        let entity = ECS::add_child_to_entity(parent_entity, child, shared);
        if let Some(rigid_body_type) = shared.entities.rigid_body_type.get(&entity).cloned() {
            let transform = Transform::from_position(start_position);
            ECS::add_rigid_body_for_entity(&entity, &rigid_body_type, &transform, shared, physics);
        }
        ECS::attach_colliders_to_entity(&entity, shared, physics);
        ECS::attach_collider_to_its_entity(&parent_entity, &entity, shared, physics);
        entity
    }

    pub fn add_entity(
        &mut self,
        parent_entity: Entity,
        child: &mut GameNodeKind,
        start_pos: (Real, Real),
    ) -> Option<Entity> {
        if let (Some(mut shared), Some(mut physics)) = (
            self.ecs.shared.try_borrow_mut(),
            self.physics.try_borrow_mut(),
        ) {
            return Some(Self::_add_entity(
                &mut shared,
                &mut physics,
                parent_entity,
                child,
                start_pos,
            ));
        }
        None
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let (Some(mut shared), Some(mut physics)) = (
            self.ecs.shared.try_borrow_mut(),
            self.physics.try_borrow_mut(),
        ) {
            let mut children_to_delete = Vec::new();
            Self::get_children_to_delete_rec(
                &mut children_to_delete,
                &entity,
                &mut shared.entities,
            );
            if let Some(rigid_body) = shared.entities.rigid_body_handle.get(&entity) {
                physics.remove_rigid_body(*rigid_body);
            }
            shared.entities.remove_entity(entity);
            shared.removed_entities.push(entity);
            for child in children_to_delete {
                shared.entities.remove_entity(child);
                shared.removed_entities.push(child);
            }
        }
    }

    pub fn get_children_to_delete_rec(
        children_to_delete: &mut Vec<Entity>,
        entity: &Entity,
        entities: &mut EntityMaps,
    ) {
        for child in entities
            .game_node_children
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Self::get_children_to_delete_rec(children_to_delete, &child, entities);
            children_to_delete.push(child);
        }
    }
}
