use flume::Sender;
use log::{debug, error};
use rapier2d::math::Real;
use std::collections::{HashMap, HashSet};
use tokio::time::Instant;

use crate::core::blueprint::character_animation::CharacterAnimation;
use crate::core::blueprint::def::{
    BlueprintService, Chunk, GameMap, Gid, LayerKind, LayerParralaxMap, Module, ModuleId,
    ResourcePath, TerrainParams,
};
use crate::core::blueprint::ecs::character_animation::Animation;
use crate::core::blueprint::ecs::def::{Entity, EntityUpdate, EntityUpdateKind};
use crate::core::blueprint::scene::def::{CollisionShape, GameNodeKind};
use crate::core::blueprint::scene::imp::build_scene_from_ecs;
use crate::core::guest::ActorId;
use crate::core::guest::{Admin, Guest, ModuleEnterSlot};
use crate::core::module::{
    create_module_communication_input, EnterFailedState, EnterSuccessState, GameSystemToGuest,
    GameSystemToGuestEvent, GuestEvent, GuestToModule, LeaveFailedState, LeaveSuccessState,
    ModuleInputSender, ModuleInstanceEvent, ModuleOutputSender,
};
use crate::core::module::{GuestInput, GuestToModuleEvent};
use crate::core::module_system::def::{
    DynamicGameModule, GuestCommunication, GuestMap, ModuleAdmin, ModuleCommunication, ModuleGuest,
};
use crate::core::module_system::error::{CreateWorldError, DestroyWorldError};
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::module_system::world::{World, WorldId};
use crate::core::{send_and_log_error, send_and_log_error_custom, LazyHashmapSet};

impl DynamicGameModule {
    pub fn create(
        instance_id: GameInstanceId,
        module: &Module,
        module_output_sender: ModuleOutputSender,
    ) -> (DynamicGameModule, ModuleInputSender) {
        let (module_input_sender, module_input_receiver) = create_module_communication_input();
        let gid_to_collision_shape_map =
            BlueprintService::generate_gid_to_shape_map(&module.resources).unwrap_or_else(|err| {
                error!("Could not load gid to collision shape map! {:?}", err);
                HashMap::new()
            });
        let mut dynamic_module = DynamicGameModule {
            world_map: HashMap::new(),
            guests: HashMap::new(),
            admins: HashMap::new(),
            guest_to_world: HashMap::new(),
            admin_to_world: LazyHashmapSet::new(),
            world_to_admin: LazyHashmapSet::new(),
            world_to_guest: LazyHashmapSet::new(),
            connected_actor_set: HashSet::new(),
            gid_to_collision_shape_map,
            module_communication: ModuleCommunication::new(
                module_input_receiver,
                module_output_sender,
            ),
            module_id: module.id.clone(),
            instance_id,
        };
        let game_maps = BlueprintService::load_all_maps_for_module(module).unwrap_or_else(|err| {
            error!("Could not load maps for module to create worlds {:?}", err);
            Vec::new()
        });
        for game_map in game_maps {
            if let Err(err) = dynamic_module.create_world(&game_map) {
                error!("Could not create world '{}': {:?}", game_map.name, err);
            }
        }

        (dynamic_module, module_input_sender)
    }

    pub fn update_gid_collision_shape_map(
        &mut self,
        gid: &Gid,
        collision_shape_option: &Option<CollisionShape>,
    ) {
        match collision_shape_option {
            None => {
                self.gid_to_collision_shape_map.remove(&gid);
            }
            Some(collision_shape) => {
                self.gid_to_collision_shape_map
                    .insert(*gid, collision_shape.clone());
            }
        }
        for world in self.world_map.values_mut() {
            if let Some(mut physics) = world.physics.try_borrow_mut() {
                world.terrain_manager.update_collision_shape(
                    gid,
                    &self.gid_to_collision_shape_map,
                    &mut physics,
                );
                Self::send_event_to_admins(
                    &world.world_id,
                    &mut self.module_communication,
                    &self.world_to_admin,
                    ModuleInstanceEvent {
                        module_id: self.module_id.clone(),
                        instance_id: self.instance_id.clone(),
                        world_id: Some(world.world_id.clone()),
                        event_type: GameSystemToGuestEvent::ShowTerrainCollisionLines(
                            world.terrain_manager.get_lines_as_vert_vec(),
                        ),
                    },
                    "Could not send terrain collision line update for collision shapes to admins!",
                );
            }
        }
    }

    pub fn create_world(&mut self, game_map: &GameMap) -> Result<WorldId, CreateWorldError> {
        if self.world_map.contains_key(&game_map.world_id) {
            return Err(CreateWorldError::DidAlreadyExist);
        }

        let new_world = World::new(game_map, &self.gid_to_collision_shape_map)?;
        self.world_map.insert(game_map.world_id.clone(), new_world);
        self.world_to_admin.init(game_map.world_id.clone());
        self.world_to_guest.init(game_map.world_id.clone());
        Ok(game_map.world_id.clone())
    }

    pub fn reset_world(&mut self, world_id: &WorldId) -> Result<(), CreateWorldError> {
        if let Some(world) = self.world_map.get_mut(world_id) {
            return world.reset();
        }

        Err(CreateWorldError::CouldNotFindWorld)
    }

    pub fn remove_script(&mut self, resource_path: &ResourcePath) {
        for world in self.world_map.values_mut() {
            world.ecs.remove_script_on_all_entities(resource_path);
        }
    }

    pub fn get_terrain_params(&self, world_id: &WorldId) -> Option<TerrainParams> {
        if let Some(world) = self.world_map.get(world_id) {
            return Some(world.terrain_manager.params.clone());
        }
        None
    }

    pub fn save_and_send_parallax_update_to_actors(
        &mut self,
        world_id: &WorldId,
        (layer_kind, (x, y)): &(LayerKind, (f32, f32)),
    ) {
        if let Some(world) = self.world_map.get_mut(world_id) {
            world
                .terrain_manager
                .layer_parallax
                .insert(layer_kind.clone(), (*x, *y));
            Self::send_event_to_actors(
                world_id,
                &mut self.module_communication,
                &self.world_to_guest,
                &self.world_to_admin,
                &self.connected_actor_set,
                ModuleInstanceEvent {
                    module_id: self.module_id.clone(),
                    instance_id: self.instance_id.clone(),
                    world_id: Some(world_id.clone()),
                    event_type: GameSystemToGuestEvent::SetParallax(
                        world
                            .terrain_manager
                            .layer_parallax
                            .clone()
                            .into_iter()
                            .map(|(layer_kind, (x, y))| (layer_kind.clone(), x, y))
                            .collect(),
                    ),
                },
                "Could not send parallax update",
            );
        }
    }

    pub fn get_parallax(&self, world_id: &WorldId) -> Option<LayerParralaxMap> {
        if let Some(world) = self.world_map.get(world_id) {
            return Some(world.terrain_manager.layer_parallax.clone());
        }
        None
    }

    pub fn destroy_world(&mut self, game_map: &GameMap) -> Result<WorldId, DestroyWorldError> {
        if self.world_to_guest.len(&game_map.world_id) > 0
            || self.world_to_admin.len(&game_map.world_id) > 0
        {
            return Err(DestroyWorldError::StillHasInhabitants);
        }
        if !self.world_map.contains_key(&game_map.world_id) {
            return Err(DestroyWorldError::DidNotExist);
        }
        self.world_map.remove(&game_map.world_id);
        self.world_to_admin.remove(&game_map.world_id);
        self.world_to_guest.remove(&game_map.world_id);
        Ok(game_map.world_id.clone())
    }

    pub fn set_actor_input(
        guest_to_world_map: &HashMap<ActorId, WorldId>,
        admin_to_world_map: &LazyHashmapSet<ActorId, WorldId>,
        world_map: &mut HashMap<WorldId, World>,
        actor_id: &ActorId,
        input: GuestInput,
    ) {
        let world_ids_of_admin_or_guest = guest_to_world_map
            .get(actor_id)
            .into_iter()
            .chain(admin_to_world_map.hashset(actor_id).into_iter().flatten());

        for world_id in world_ids_of_admin_or_guest {
            if let Some(world) = world_map.get_mut(world_id) {
                if let Some(mut actor_api) = world.actor_api.try_borrow_mut() {
                    actor_api.set_actor_input(*actor_id, input.clone());
                }
            }
        }
    }

    fn set_resources_loaded(
        guests: &mut GuestMap,
        connected_actors_set: &mut HashSet<ActorId>,
        guest_id: &ActorId,
    ) {
        if let Some(guest) = guests.get_mut(guest_id) {
            connected_actors_set.insert(*guest_id);
            guest.guest_com.resources_loaded = true;
        }
    }

    pub fn update(&mut self, module: &Module) {
        self.handle_guest_events(module);
        self.send_scope_updates_to_admins();
        for world in self.world_map.values_mut() {
            world.update();

            let position_updates = Self::get_position_updates(world);
            if !position_updates.is_empty() {
                let update_position_event = ModuleInstanceEvent {
                    world_id: None,
                    module_id: self.module_id.clone(),
                    instance_id: self.instance_id.clone(),
                    event_type: GameSystemToGuestEvent::PositionEvent(position_updates),
                };
                Self::send_event_to_actors(
                    &world.world_id,
                    &mut self.module_communication,
                    &self.world_to_guest,
                    &self.world_to_admin,
                    &self.connected_actor_set,
                    update_position_event,
                    "Could not send entity update",
                );
            }

            let gid_updates = Self::get_gid_updates(world);
            if !gid_updates.is_empty() {
                for (entity, gid) in gid_updates {
                    let update_position_event = ModuleInstanceEvent {
                        world_id: None,
                        module_id: self.module_id.clone(),
                        instance_id: self.instance_id.clone(),
                        event_type: GameSystemToGuestEvent::UpdateEntity(EntityUpdate {
                            id: entity,
                            kind: EntityUpdateKind::Gid(gid),
                        }),
                    };
                    Self::send_event_to_actors(
                        &world.world_id,
                        &mut self.module_communication,
                        &self.world_to_guest,
                        &self.world_to_admin,
                        &self.connected_actor_set,
                        update_position_event,
                        "Could not send entity update",
                    );
                }
            }
        }
    }

    fn send_scope_updates_to_admins(&mut self) {
        for world in self.world_map.values_mut() {
            if self.world_to_admin.len(&world.world_id) > 0 {
                for (entity, game_node_script) in world.ecs.entity_scripts.iter_mut() {
                    if let Some(scope_update) = game_node_script.update_scope_cache() {
                        let scope_update_event = ModuleInstanceEvent {
                            world_id: None,
                            module_id: self.module_id.clone(),
                            instance_id: self.instance_id.clone(),
                            event_type: GameSystemToGuestEvent::UpdateEntity(EntityUpdate {
                                id: *entity,
                                kind: EntityUpdateKind::SetScriptScope(scope_update),
                            }),
                        };
                        Self::send_event_to_admins(
                            &world.world_id,
                            &mut self.module_communication,
                            &self.world_to_admin,
                            scope_update_event,
                            "Could not send scope update to admins",
                        );
                    }
                }
            }
        }
    }

    pub fn get_position_updates(world: &mut World) -> Vec<(Entity, Real, Real, Real)> {
        if let Some(mut shared) = world.ecs.shared.try_borrow_mut() {
            let entities = &mut shared.entities;
            let transforms = &mut entities.transforms;
            entities
                .dirty
                .drain()
                .filter_map(|(entity, dirty)| {
                    if dirty {
                        transforms.get(&entity).map(|transform| {
                            (
                                entity,
                                transform.position.0,
                                transform.position.1,
                                transform.rotation,
                            )
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_gid_updates(world: &mut World) -> Vec<(Entity, Gid)> {
        if let Some(mut shared) = world.ecs.shared.try_borrow_mut() {
            let entities = &mut shared.entities;
            let render_gid = &mut entities.render_gid;
            entities
                .view_dirty
                .drain()
                .filter_map(|(entity, dirty)| {
                    if dirty {
                        render_gid.get(&entity).map(|gid| (entity, *gid))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn actor_disconnected(&mut self, actor_id: &ActorId) {
        if let Some(guest) = self.guests.get_mut(actor_id) {
            guest.guest_com.connected = false;
            if let Some(world_id) = &guest.world_id {
                if let Some(world) = self.world_map.get_mut(world_id) {
                    world.actor_disconnected(*actor_id);
                }
            }
        }
        if let Some(admin) = self.admins.get_mut(actor_id) {
            admin.connected = false;
            if let Some(world_ids) = self.admin_to_world.hashset(actor_id) {
                for world_id in world_ids {
                    if let Some(world) = self.world_map.get_mut(world_id) {
                        world.actor_disconnected(*actor_id);
                    }
                }
            }
        }
    }

    pub fn apply_admin_entity_update(&mut self, world_id: &WorldId, entity_update: EntityUpdate) {
        if let Some(world) = self.world_map.get_mut(world_id) {
            world.apply_admin_entity_update(entity_update.clone());

            let entity_update_event = ModuleInstanceEvent {
                world_id: None,
                module_id: self.module_id.clone(),
                instance_id: self.instance_id.clone(),
                event_type: GameSystemToGuestEvent::UpdateEntity(entity_update),
            };
            Self::send_event_to_actors(
                world_id,
                &mut self.module_communication,
                &self.world_to_guest,
                &self.world_to_admin,
                &self.connected_actor_set,
                entity_update_event,
                "Could not send entity update",
            );
        }
    }

    pub fn remove_entity(&mut self, world_id: &WorldId, entity: Entity) {
        if let Some(world) = self.world_map.get_mut(world_id) {
            world.remove_entity(entity);
        }
        let entity_removed_event = ModuleInstanceEvent {
            world_id: None,
            module_id: self.module_id.clone(),
            instance_id: self.instance_id.clone(),
            event_type: GameSystemToGuestEvent::RemoveEntity(entity),
        };
        Self::send_event_to_actors(
            world_id,
            &mut self.module_communication,
            &self.world_to_guest,
            &self.world_to_admin,
            &self.connected_actor_set,
            entity_removed_event,
            "Could not send entity remove event",
        );
    }

    pub fn add_entity(
        &mut self,
        world_id: &WorldId,
        parent_entity: Entity,
        game_node: GameNodeKind,
    ) {
        if let Some(world) = self.world_map.get_mut(world_id) {
            if let Some(entity) = world.add_entity(parent_entity, &game_node) {
                if let Some(game_node) =
                    GameNodeKind::get_game_node_kind_from_ecs(&entity, &world.ecs)
                {
                    let add_entity_event = ModuleInstanceEvent {
                        world_id: None,
                        module_id: self.module_id.clone(),
                        instance_id: self.instance_id.clone(),
                        event_type: GameSystemToGuestEvent::AddEntity(parent_entity, game_node),
                    };

                    Self::send_event_to_actors(
                        world_id,
                        &mut self.module_communication,
                        &self.world_to_guest,
                        &self.world_to_admin,
                        &self.connected_actor_set,
                        add_entity_event,
                        "Could not send entity add event",
                    );
                } else {
                    error!("Could not create game node from entity!!");
                }
            } else {
                error!("Could not add entity to world {:?}", world_id);
            }
        } else {
            error!("Could not find world {:?}", world_id);
        }
    }

    pub fn update_character_animation(&mut self, character_animation: &CharacterAnimation) {
        for world in self.world_map.values_mut() {
            if let Some(mut shared) = world.ecs.shared.try_borrow_mut() {
                for c_a in shared.entities.character_animation.values_mut() {
                    if c_a.blueprint.id == character_animation.id {
                        if !character_animation.states.contains_key(&c_a.current_state) {
                            c_a.current_state = character_animation.start_state;
                        }
                        c_a.states.clear();
                        for (state_id, state) in &character_animation.states {
                            c_a.states
                                .insert(*state_id, Animation::new(state.frames.clone()));
                        }
                    }
                }
            }
        }
    }

    pub fn update_world_map(&mut self, world_id: &WorldId, layer_kind: &LayerKind, chunk: &Chunk) {
        if let Some(world) = self.world_map.get_mut(world_id) {
            if let Some(mut physics) = world.physics.try_borrow_mut() {
                world.terrain_manager.write_chunk(
                    chunk,
                    layer_kind,
                    &self.gid_to_collision_shape_map,
                    &mut physics,
                );

                let terrain_update = ModuleInstanceEvent {
                    world_id: None,
                    module_id: self.module_id.clone(),
                    instance_id: self.instance_id.clone(),
                    event_type: GameSystemToGuestEvent::ShowTerrain(vec![(
                        layer_kind.clone(),
                        vec![chunk.clone()],
                    )]),
                };
                Self::send_event_to_actors(
                    world_id,
                    &mut self.module_communication,
                    &self.world_to_guest,
                    &self.world_to_admin,
                    &self.connected_actor_set,
                    terrain_update,
                    "Could not send terrain update",
                );
                Self::send_event_to_admins(
                    world_id,
                    &mut self.module_communication,
                    &self.world_to_admin,
                    ModuleInstanceEvent {
                        module_id: self.module_id.clone(),
                        instance_id: self.instance_id.clone(),
                        world_id: Some(world_id.clone()),
                        event_type: GameSystemToGuestEvent::ShowTerrainCollisionLines(
                            world.terrain_manager.get_lines_as_vert_vec(),
                        ),
                    },
                    "Could not send terrain collision line update to admins!",
                );
            }
        } else {
            error!("Could not update chunk in world {:?}", world_id);
        }
    }

    fn send_event_to_admins(
        world_id: &WorldId,
        module_communication: &mut ModuleCommunication,
        world_to_admin: &LazyHashmapSet<WorldId, ActorId>,
        event: ModuleInstanceEvent<GameSystemToGuestEvent>,
        custom_error_msg: &str,
    ) {
        if let Some(admin_ids) = world_to_admin.hashset(world_id) {
            let mut admin_event = event;
            admin_event.world_id = Some(world_id.clone());
            for admin_id in admin_ids {
                send_and_log_error_custom(
                    &mut module_communication
                        .output_sender
                        .game_system_to_guest_sender,
                    GuestEvent {
                        guest_id: *admin_id,
                        event_type: admin_event.clone(),
                    },
                    custom_error_msg,
                );
            }
        }
    }

    fn send_event_to_actors(
        world_id: &WorldId,
        module_communication: &mut ModuleCommunication,
        world_to_guest: &LazyHashmapSet<WorldId, ActorId>,
        world_to_admin: &LazyHashmapSet<WorldId, ActorId>,
        connected_actors_set: &HashSet<ActorId>,
        event: ModuleInstanceEvent<GameSystemToGuestEvent>,
        custom_error_msg: &str,
    ) {
        if let (Some(actor_id), Some(admin_ids)) = (
            world_to_guest.hashset(world_id),
            world_to_admin.hashset(world_id),
        ) {
            let send_event_update =
                &mut |actor_id: ActorId, update: ModuleInstanceEvent<GameSystemToGuestEvent>| {
                    send_and_log_error_custom(
                        &mut module_communication
                            .output_sender
                            .game_system_to_guest_sender,
                        GuestEvent {
                            guest_id: actor_id,
                            event_type: update,
                        },
                        custom_error_msg,
                    );
                };
            for guest_id in actor_id {
                if connected_actors_set.contains(guest_id) {
                    send_event_update(*guest_id, event.clone());
                }
            }
            let mut admin_event = event;
            admin_event.world_id = Some(world_id.clone());
            for admin_id in admin_ids {
                if connected_actors_set.contains(admin_id) {
                    send_event_update(*admin_id, admin_event.clone());
                }
            }
        }
    }

    pub fn actor_reconnected(&mut self, actor_id: &ActorId) {
        if let Some(guest) = self.guests.get_mut(actor_id) {
            guest.guest_com.connected = true;
            if let Some(world_id) = &guest.world_id {
                if let Some(world) = self.world_map.get_mut(world_id) {
                    world.actor_reconnected(*actor_id);
                }
            }
        }
        if let Some(admin) = self.admins.get_mut(actor_id) {
            admin.connected = true;
            if let Some(world_ids) = self.admin_to_world.hashset(actor_id) {
                for world_id in world_ids {
                    if let Some(world) = self.world_map.get_mut(world_id) {
                        world.actor_reconnected(*actor_id);
                    }
                }
            }
        }
    }

    fn handle_guest_events(&mut self, module: &Module) {
        for event in self
            .module_communication
            .input_receiver
            .guest_to_module_receiver
            .drain()
        {
            let GuestToModule {
                guest_id,
                event_type,
            } = event;
            match event_type.event_type {
                GuestToModuleEvent::ControlInput(input) => {
                    Self::set_actor_input(
                        &self.guest_to_world,
                        &self.admin_to_world,
                        &mut self.world_map,
                        &guest_id,
                        input,
                    );
                }
                GuestToModuleEvent::GameSetupDone => {
                    Self::set_resources_loaded(
                        &mut self.guests,
                        &mut self.connected_actor_set,
                        &guest_id,
                    );
                    if let Some(world_id) = self.guest_to_world.get(&guest_id) {
                        Self::send_initial_world_events(
                            &mut self
                                .module_communication
                                .output_sender
                                .game_system_to_guest_sender,
                            &self.world_map,
                            self.instance_id.clone(),
                            guest_id,
                            world_id,
                            module.id.clone(),
                            false,
                            true,
                        );
                    }
                }
                GuestToModuleEvent::WantToChangeModule(_exit_slot) => {
                    debug!("WantToChangeModule not implemented!");
                }
            }
        }
    }

    pub fn send_initial_world_events_admin(
        &mut self,
        admin_id: ActorId,
        world_id: &WorldId,
        module_id: ModuleId,
        send_terrain: bool,
    ) {
        Self::send_initial_world_events(
            &mut self
                .module_communication
                .output_sender
                .game_system_to_guest_sender,
            &self.world_map,
            self.instance_id.clone(),
            admin_id,
            world_id,
            module_id,
            true,
            send_terrain,
        );
    }

    pub fn send_initial_world_events(
        sender: &mut Sender<GameSystemToGuest>,
        world_map: &HashMap<WorldId, World>,
        instance_id: GameInstanceId,
        actor_id: ActorId,
        world_id: &WorldId,
        module_id: ModuleId,
        is_admin: bool,
        send_terrain: bool,
    ) {
        if send_terrain {
            if let Some(initial_terrain_event) = Self::get_initial_terrain_event(
                world_map,
                module_id.clone(),
                instance_id.clone(),
                world_id,
                is_admin,
            ) {
                send_and_log_error(
                    sender,
                    GuestEvent {
                        guest_id: actor_id,
                        event_type: initial_terrain_event,
                    },
                );
            }
        }
        if let Some(world) = world_map.get(world_id) {
            if is_admin {
                send_and_log_error(
                    sender,
                    GuestEvent {
                        guest_id: actor_id,
                        event_type: ModuleInstanceEvent {
                            module_id: module_id.clone(),
                            instance_id: instance_id.clone(),
                            world_id: Some(world_id.clone()),
                            event_type: GameSystemToGuestEvent::ShowTerrainCollisionLines(
                                world.terrain_manager.get_lines_as_vert_vec(),
                            ),
                        },
                    },
                );

                Self::send_current_script_scopes(sender, &instance_id, actor_id, &module_id, world);
            }
            if let Some(scene) = build_scene_from_ecs(&world.ecs) {
                send_and_log_error(
                    sender,
                    GuestEvent {
                        guest_id: actor_id,
                        event_type: ModuleInstanceEvent {
                            module_id,
                            instance_id,
                            world_id: if is_admin {
                                Some(world_id.clone())
                            } else {
                                None
                            },
                            event_type: GameSystemToGuestEvent::ShowScene(scene),
                        },
                    },
                );
            } else {
                error!(
                    "Was not able to get scene from world! world-id: {:?}",
                    world_id
                );
            }
        }
    }

    pub fn send_current_script_scopes(
        sender: &mut Sender<GameSystemToGuest>,
        instance_id: &GameInstanceId,
        actor_id: ActorId,
        module_id: &ModuleId,
        world: &World,
    ) {
        for (entity, game_node_script) in world.ecs.entity_scripts.iter() {
            let initial_scope = game_node_script.scope_cache.clone();
            let scope_update_event = ModuleInstanceEvent {
                world_id: Some(world.world_id.clone()),
                module_id: module_id.clone(),
                instance_id: instance_id.clone(),
                event_type: GameSystemToGuestEvent::UpdateEntity(EntityUpdate {
                    id: *entity,
                    kind: EntityUpdateKind::SetScriptScope(initial_scope),
                }),
            };
            send_and_log_error(
                sender,
                GuestEvent {
                    guest_id: actor_id,
                    event_type: scope_update_event,
                },
            );
        }
    }

    pub fn get_initial_terrain_event(
        world_map: &HashMap<WorldId, World>,
        module_id: ModuleId,
        instance_id: GameInstanceId,
        world_id: &WorldId,
        set_world: bool,
    ) -> Option<ModuleInstanceEvent<GameSystemToGuestEvent>> {
        Self::get_all_terrain(world_map, world_id).map(|terrain| ModuleInstanceEvent {
            module_id,
            instance_id,
            world_id: if set_world {
                Some(world_id.clone())
            } else {
                None
            },
            event_type: GameSystemToGuestEvent::ShowTerrain(terrain),
        })
    }

    pub fn let_admin_enter(
        &mut self,
        admin: &Admin,
        world_id: WorldId,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        self.world_to_admin.insert_entry(world_id.clone(), admin.id);
        debug!("Admin entered world: {:?}", world_id);
        self.admin_to_world.insert_entry(admin.id, world_id.clone());
        debug!("admin to world: {:?}", self.admin_to_world);
        self.admins.entry(admin.id).or_insert(ModuleAdmin {
            id: admin.id,
            last_input_time: Instant::now(),
            connected: true,
            resources_loaded: false,
        });

        if let Some(world) = self.world_map.get_mut(&world_id) {
            world.actor_joined_world(admin.id);
        }

        Ok(EnterSuccessState::Entered)
    }

    pub fn let_admin_leave(
        &mut self,
        admin: &Admin,
        world_id: WorldId,
    ) -> Result<LeaveSuccessState, LeaveFailedState> {
        self.admin_to_world.remove_entry(&admin.id, &world_id);
        self.world_to_admin.remove_entry(&world_id, &admin.id);
        if self.admin_to_world.len(&admin.id) == 0 {
            self.admins.remove(&admin.id);
            self.admin_to_world.remove(&admin.id);
        }
        if let Some(world) = self.world_map.get_mut(&world_id) {
            world.actor_left_world(admin.id);
        }

        Ok(LeaveSuccessState::Left)
    }

    pub fn try_enter(
        &mut self,
        guest: &Guest,
        main_world_id: WorldId,
        _module_enter_slot: &ModuleEnterSlot,
    ) -> Result<EnterSuccessState, EnterFailedState> {
        debug!("Guest entering world with id: {:?}", main_world_id);
        self.guest_to_world.insert(guest.id, main_world_id.clone());
        self.world_to_guest
            .insert_entry(main_world_id.clone(), guest.id);
        self.guests.insert(
            guest.id,
            ModuleGuest {
                id: guest.id,
                guest_com: GuestCommunication {
                    connected: true,
                    resources_loaded: false,
                },
                last_input_time: Instant::now(),
                world_id: Some(main_world_id.clone()),
            },
        );

        if let Some(world) = self.world_map.get_mut(&main_world_id) {
            world.actor_joined_world(guest.id);
        }

        Ok(EnterSuccessState::Entered)
    }

    pub fn try_leave(&mut self, _guest: &Guest) -> Result<LeaveSuccessState, LeaveFailedState> {
        Ok(LeaveSuccessState::Left)
    }

    pub fn get_all_terrain(
        world_map: &HashMap<WorldId, World>,
        world_id: &WorldId,
    ) -> Option<Vec<(LayerKind, Vec<Chunk>)>> {
        world_map.get(world_id).map(|world| {
            world
                .terrain_manager
                .layer_data
                .iter()
                .map(|(a, b)| (a.clone(), b.values().cloned().collect()))
                .collect()
        })
    }
}
