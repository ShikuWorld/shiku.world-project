use std::collections::HashMap;

use rapier2d::prelude::*;
use rhai::{Dynamic, Engine, FuncRegistration, Module as RhaiModule};

use crate::core::blueprint::def::{GameMap, Gid, TerrainParams};
use crate::core::blueprint::ecs::def::{
    ECSShared, Entity, EntityMaps, EntityUpdate, EntityUpdateKind, ECS,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{CollisionShape, GameNodeKind, Transform};
use crate::core::guest::ActorId;
use crate::core::module::GuestInput;
use crate::core::module_system::error::CreateWorldError;
use crate::core::module_system::terrain_manager::TerrainManager;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::ApiShare;

pub type WorldId = String;

pub struct World {
    pub world_id: WorldId,
    pub physics: ApiShare<RapierSimulation>,
    pub actor_api: ApiShare<ActorApi>,
    pub terrain_manager: TerrainManager,
    pub ecs: ECS,
    pub script_engine: Engine,
}

pub struct ActorApi {
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
        let mut ecs = ECS::from(&world_scene);
        let mut physics = RapierSimulation::new();
        let terrain_manager = TerrainManager::new(
            TerrainParams {
                chunk_size: game_map.chunk_size,
                tile_height: game_map.tile_height,
                tile_width: game_map.tile_width,
            },
            game_map.terrain.clone(),
            collision_shape_map,
            &mut physics,
        );
        Self::init_physics_simulation_from_ecs(&mut ecs, &mut physics);

        let mut script_engine = Engine::new();
        let physics_share = ApiShare::new(physics);
        Self::setup_physics_scripting_api(&mut script_engine, &physics_share, &mut ecs);
        let actor_api = ApiShare::new(ActorApi {
            actor_inputs: HashMap::new(),
        });
        Self::setup_input_scripting_api(&mut script_engine, &actor_api);
        Self::call_init_func_on_game_nodes(&script_engine, &mut ecs);
        Ok(World {
            world_id: game_map.world_id.clone(),
            physics: physics_share,
            actor_api,
            terrain_manager,
            ecs,
            script_engine,
        })
    }

    pub fn actor_joined_world(&mut self, actor_id: &ActorId) {
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call_actor_joined(&self.script_engine, actor_id);
        }
    }

    pub fn actor_left_world(&mut self, actor_id: &ActorId) {
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call_actor_left(&self.script_engine, actor_id);
        }
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

    fn setup_input_scripting_api(engine: &mut Engine, actor_api_share: &ApiShare<ActorApi>) {
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
        engine.register_static_module("shiku::input", module.into());
    }

    pub fn update(&mut self) {
        if let Some(mut physics) = self.physics.try_borrow_mut() {
            physics.update();
            if let Some(mut shared) = self.ecs.shared.try_borrow_mut() {
                Self::update_positions(&mut physics, &mut shared);
            }
        }
        for game_node_script in self.ecs.entity_scripts.values_mut() {
            game_node_script.call_update(&self.script_engine);
        }
        self.ecs.update();
    }

    fn update_positions(physics: &mut RapierSimulation, shared: &mut ECSShared) {
        for (entity, rigid_body_handle) in shared.entities.rigid_body_handle.iter() {
            if let Some(transform) = shared.entities.transforms.get_mut(entity) {
                let (x, y, r) = physics.get_rigid_body_translation(*rigid_body_handle);
                if transform.position.0 != x || transform.position.1 != y || transform.rotation != r
                {
                    transform.position = (x, y);
                    transform.rotation = r;
                    shared.entities.dirty.insert(*entity, true);
                }
            }
        }
    }

    fn call_init_func_on_game_nodes(script_engine: &Engine, ecs: &mut ECS) {
        for game_node_script in ecs.entity_scripts.values_mut() {
            game_node_script.call_init(script_engine);
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
            if let Some(rigid_body_handle) =
                shared.entities.rigid_body_handle.get(&entity_update.id)
            {
                let entity = &entity_update.id;
                match &entity_update.kind {
                    EntityUpdateKind::Transform(transform) => {
                        physics.set_translation_and_rotation_for_rigid_body(
                            Vector::new(transform.position.0, transform.position.1),
                            transform.rotation,
                            *rigid_body_handle,
                        );
                    }
                    EntityUpdateKind::PositionRotation((x, y, r)) => {
                        physics.set_translation_and_rotation_for_rigid_body(
                            Vector::new(*x, *y),
                            *r,
                            *rigid_body_handle,
                        );
                    }
                    EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                        physics.remove_rigid_body(*rigid_body_handle);
                        shared.entities.rigid_body_handle.remove(entity);

                        shared
                            .entities
                            .rigid_body_type
                            .insert(*entity, rigid_body_type.clone());
                        let transform = shared
                            .entities
                            .transforms
                            .get(entity)
                            .cloned()
                            .unwrap_or_default();
                        ECS::add_rigid_body_for_entity(
                            entity,
                            rigid_body_type,
                            &transform,
                            &mut shared,
                            &mut physics,
                        );
                        ECS::attach_colliders_to_entity(entity, &mut shared, &mut physics);
                    }
                    EntityUpdateKind::Gid(_)
                    | EntityUpdateKind::Name(_)
                    | EntityUpdateKind::UpdateScriptScope(_, _)
                    | EntityUpdateKind::SetScriptScope(_)
                    | EntityUpdateKind::InstancePath(_)
                    | EntityUpdateKind::ScriptPath(_) => {
                        ECS::apply_entity_update_s(
                            &mut self.ecs.entity_scripts,
                            &mut shared,
                            entity_update,
                            &self.script_engine,
                        );
                    }
                }
            } else {
                ECS::apply_entity_update_s(
                    &mut self.ecs.entity_scripts,
                    &mut shared,
                    entity_update,
                    &self.script_engine,
                );
            }
        }
    }

    pub fn add_entity(&mut self, parent_entity: Entity, child: &GameNodeKind) -> Option<Entity> {
        if let (Some(mut shared), Some(mut physics)) = (
            self.ecs.shared.try_borrow_mut(),
            self.physics.try_borrow_mut(),
        ) {
            let entity =
                ECS::add_child_to_entity(parent_entity, child, &mut shared, &self.script_engine);
            if let Some(rigid_body_type) = shared.entities.rigid_body_type.get(&entity).cloned() {
                let transform = Transform::default();
                ECS::add_rigid_body_for_entity(
                    &entity,
                    &rigid_body_type,
                    &transform,
                    &mut shared,
                    &mut physics,
                );
            }
            ECS::attach_colliders_to_entity(&entity, &mut shared, &mut physics);
            ECS::attach_collider_to_its_entity(&parent_entity, &entity, &mut shared, &mut physics);
            return Some(entity);
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
