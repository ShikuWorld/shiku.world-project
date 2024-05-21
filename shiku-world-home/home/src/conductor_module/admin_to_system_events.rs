use std::collections::HashSet;
use std::path::PathBuf;

use flume::Sender;
use log::{debug, error};
use uuid::Uuid;

use crate::conductor_module::blueprint_helper::{
    bring_polygon_in_clockwise_order, save_and_send_conductor_update,
};
use crate::conductor_module::def::{
    ConductorModule, ModuleCommunicationMap, ModuleMap, ResourceToModuleMap,
};
use crate::conductor_module::game_instances::{
    create_game_instance_manager, remove_game_instance_manager,
};
use crate::core::blueprint::def::{
    BlueprintResource, BlueprintService, Conductor, ResourceKind, ResourceLoaded, Tileset,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::CollisionShape;
use crate::core::guest::{ActorId, Admin};
use crate::core::module::{
    AdminToSystemEvent, CommunicationEvent, EditorEvent, GuestToModuleEvent, SceneNodeUpdate,
    TilesetUpdate,
};
use crate::core::module_system::def::DynamicGameModule;
use crate::core::{log_result_error, send_and_log_error};
use crate::resource_module::def::{ResourceBundle, ResourceEvent, ResourceModule};
use crate::webserver_module::def::WebServerModule;

pub async fn handle_admin_to_system_event(
    module_communication_map: &mut ModuleCommunicationMap,
    web_server_module: &mut WebServerModule,
    resource_module: &mut ResourceModule,
    module_map: &mut ModuleMap,
    resource_to_module_map: &mut ResourceToModuleMap,
    system_to_admin_communication_sender: &mut Sender<(ActorId, CommunicationEvent)>,
    admin: &Admin,
    event: AdminToSystemEvent,
) {
    let mut send_communication_event = |event: CommunicationEvent| {
        send_and_log_error(system_to_admin_communication_sender, (admin.id, event));
    };

    let mut send_editor_event = |event: EditorEvent| {
        send_communication_event(CommunicationEvent::EditorEvent(event));
    };

    match event {
        AdminToSystemEvent::ControlInput(module_id, instance_id, guest_input) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                if let Some(instance) = module.game_instances.get_mut(&instance_id) {
                    DynamicGameModule::set_actor_input(
                        &instance.dynamic_module.guest_to_world,
                        &instance.dynamic_module.admin_to_world,
                        &mut instance.dynamic_module.world_map,
                        &admin.id,
                        guest_input,
                    );
                }
            }
        }
        AdminToSystemEvent::WorldInitialized(module_id, instance_id, world_id) => {
            if let Some(instance) = module_map
                .get_mut(&module_id)
                .and_then(|module| module.game_instances.get_mut(&instance_id))
            {
                if let Some(module_admin) = instance.dynamic_module.admins.get_mut(&admin.id) {
                    module_admin.resources_loaded = true;
                }
                instance.dynamic_module.send_initial_world_events_admin(
                    admin.id,
                    &world_id,
                    module_id.clone(),
                );
            }
        }
        AdminToSystemEvent::OpenInstance(module_id) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                module.create_new_game_instance();
            }
        }
        AdminToSystemEvent::StartInspectingWorld(module_id, game_instance_id, world_id) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                match module.let_admin_into_instance(
                    admin,
                    game_instance_id.clone(),
                    world_id.clone(),
                ) {
                    Ok(_) => {
                        resource_module
                            .activate_module_resource_updates(module_id.clone(), &admin.id);
                        match resource_module.get_active_resources_for_module(&module_id, &admin.id)
                        {
                            Ok(assets) => {
                                if let Some(terrain_params) = module.get_terrain_params_for_admin(
                                    &admin.id,
                                    &game_instance_id,
                                    &world_id,
                                ) {
                                    match BlueprintService::load_module_tilesets(
                                        &module.module_blueprint.resources,
                                    ) {
                                        Ok(tilesets) => {
                                            send_communication_event(
                                                CommunicationEvent::PrepareGame(
                                                    module_id.clone(),
                                                    game_instance_id.clone(),
                                                    Some(world_id.clone()),
                                                    ResourceBundle {
                                                        name: "Default".into(),
                                                        assets,
                                                    },
                                                    terrain_params,
                                                    tilesets,
                                                    module.module_blueprint.gid_map.clone(),
                                                ),
                                            );
                                        }
                                        Err(err) => {
                                            error!("Could not load tilesets for module! {:?}", err)
                                        }
                                    }
                                } else {
                                    error!(
                                        "Could not get terrain params to inspect! {:?} {:?} {:?}",
                                        module_id, game_instance_id, world_id
                                    );
                                }
                            }
                            Err(err) => {
                                error!("Could not send prepare game, no resources?! {:?}", err)
                            }
                        }
                    }
                    Err(err) => error!("Could not get admin into instance/map {:?}", err),
                }
            } else {
                error!(
                    "module not found to inspect...? {:?} {:?} {:?}",
                    module_id, game_instance_id, world_id
                );
            }
        }
        AdminToSystemEvent::StopInspectingWorld(module_id, game_instance_id, world_id) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                match module.let_admin_leave_instance(
                    admin,
                    game_instance_id.clone(),
                    world_id.clone(),
                ) {
                    Ok(_) => {
                        if let Err(err) = resource_module
                            .disable_module_resource_updates(module_id.clone(), &admin.id)
                        {
                            error!("Could not unregister from resource updates! {:?}", err);
                        }
                        send_communication_event(CommunicationEvent::UnloadGame(
                            module_id,
                            game_instance_id,
                            Some(world_id),
                        ));
                    }
                    Err(err) => error!("Could not let admin leave instance/map {:?}", err),
                }
            }
        }
        AdminToSystemEvent::UpdateMap(map_update) => {
            let map_path = format!("{}/{}.map.json", map_update.resource_path, map_update.name);
            match Blueprint::load_map(PathBuf::from(map_path)) {
                Ok(mut map) => {
                    if let Some((layer, chunk)) = map_update.chunk.clone() {
                        map.set_chunk(layer, chunk);
                    }
                    match Blueprint::save_map(&map) {
                        Ok(()) => {
                            if let (Some(module), Some((layer_kind, chunk))) =
                                (module_map.get_mut(&map.module_id), &map_update.chunk)
                            {
                                module.update_world_map(&map.world_id, layer_kind, chunk);
                            }
                            send_editor_event(EditorEvent::UpdatedMap(map_update));
                        }
                        Err(err) => {
                            error!("Could not update map {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Could not load map {:?}", err);
                }
            }
        }
        AdminToSystemEvent::DeleteMap(module_id, map) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                match Blueprint::delete_map(&map) {
                    Ok(()) => {
                        let map_path = format!("{}/{}", map.resource_path, map.name);
                        module
                            .module_blueprint
                            .resources
                            .retain(|r| r.path != map_path);

                        match Blueprint::save_module(&module.module_blueprint) {
                            Ok(()) => {
                                send_editor_event(EditorEvent::UpdatedModule(
                                    module_id,
                                    module.module_blueprint.clone(),
                                ));
                                send_editor_event(EditorEvent::DeletedMap(map));
                            }
                            Err(err) => {
                                error!("Could not save module {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Could not create map {:?}", err);
                    }
                }
            }
        }
        AdminToSystemEvent::CreateMap(module_id, mut map) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                map.world_id = Uuid::new_v4().to_string();
                match Blueprint::create_map(&map) {
                    Ok(()) => {
                        module.module_blueprint.resources.push(BlueprintResource {
                            file_name: format!("{}.map.json", map.name),
                            dir: map.resource_path.clone(),
                            path: format!("{}/{}.map.json", map.resource_path, map.name),
                            kind: ResourceKind::Map,
                        });
                        match Blueprint::save_module(&module.module_blueprint) {
                            Ok(()) => {
                                module
                                    .create_world(&map)
                                    .values()
                                    .filter(|f| f.is_err())
                                    .for_each(|err| error!("{:?}", err));
                                send_editor_event(EditorEvent::UpdatedModule(
                                    module_id,
                                    module.module_blueprint.clone(),
                                ));
                                send_editor_event(EditorEvent::SetMap(map));
                            }
                            Err(err) => {
                                error!("Could not save module {:?}", err);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Could not create map {:?}", err);
                    }
                }
            }
        }
        AdminToSystemEvent::GetResource(path) => {
            match BlueprintService::load_resource_by_path(&path) {
                ResourceLoaded::Tileset(tileset) => {
                    send_editor_event(EditorEvent::SetTileset(tileset));
                }
                ResourceLoaded::Scene(scene) => {
                    send_editor_event(EditorEvent::SetScene(scene));
                }
                ResourceLoaded::Map(map) => {
                    send_editor_event(EditorEvent::SetMap(map));
                }
                ResourceLoaded::Script(script) => {
                    send_editor_event(EditorEvent::SetScript(script));
                }
                ResourceLoaded::Unknown => {
                    debug!("unknown resource {:?}", path);
                }
            }
        }
        AdminToSystemEvent::BrowseFolder(path) => match BlueprintService::browse_directory(path) {
            Ok(result) => {
                send_editor_event(EditorEvent::DirectoryInfo(result));
            }
            Err(err) => {
                error!("Could not browse directory {:?}", err);
            }
        },
        AdminToSystemEvent::SetMainDoorStatus(status) => {
            debug!("Setting main door status");
            web_server_module.set_main_door_status(status).await;
        }
        AdminToSystemEvent::SetBackDoorStatus(status) => {
            debug!("Setting back door status");
            web_server_module.set_back_door_status(status).await;
        }
        AdminToSystemEvent::ProviderLoggedIn(_) => {
            error!("Admin should already be logged in!")
        }
        AdminToSystemEvent::Ping => {}
        AdminToSystemEvent::UpdateConductor(conductor) => {
            save_and_send_conductor_update(conductor, &mut send_editor_event);
        }
        AdminToSystemEvent::LoadEditorData => {
            match BlueprintService::load_conductor_blueprint() {
                Ok(conductor) => {
                    send_editor_event(EditorEvent::UpdatedConductor(conductor));
                }
                Err(err) => {
                    error!("Could not load conductor! {:?}", err);
                }
            }
            match Blueprint::get_all_modules() {
                Ok(modules) => {
                    send_editor_event(EditorEvent::Modules(modules));
                }
                Err(err) => {
                    error!("Could not retrieve modules! {:?}", err);
                }
            }
            send_editor_event(EditorEvent::ModuleInstances(
                module_map
                    .values()
                    .map(|m| {
                        (
                            m.module_blueprint.id.clone(),
                            m.game_instances.values().map(|g| g.id.clone()).collect(),
                        )
                    })
                    .collect(),
            ));
        }
        AdminToSystemEvent::CreateTileset(tileset) => match Blueprint::create_tileset(&tileset) {
            Ok(()) => {
                send_editor_event(EditorEvent::CreatedTileset(tileset));
            }
            Err(err) => error!("Could not create tileset: {:?}", err),
        },
        AdminToSystemEvent::UpdateTileset(resource_path, tileset_update) => {
            if let Ok(mut tileset) = Blueprint::load_tileset(resource_path.clone().into()) {
                match tileset_update {
                    TilesetUpdate::UpdateCollisionShape(tile_id, mut collision_shape) => {
                        let tile = tileset.tiles.entry(tile_id).or_default();
                        if let CollisionShape::Polygon(ref mut vertices) = &mut collision_shape {
                            bring_polygon_in_clockwise_order(vertices);
                        }
                        tile.collision_shape = Some(collision_shape);
                        for m in module_map.values_mut() {
                            if let Some(gid_start) =
                                m.module_blueprint
                                    .gid_map
                                    .0
                                    .iter()
                                    .find_map(|(r_path, gid)| {
                                        if *r_path == resource_path {
                                            Some(gid)
                                        } else {
                                            None
                                        }
                                    })
                            {
                                m.update_gid_collision_shape_map(
                                    &(gid_start + tile_id),
                                    &tile.collision_shape,
                                )
                            }
                        }
                    }
                    TilesetUpdate::RemoveCollisionShape(gid) => {
                        let tile = tileset.tiles.entry(gid).or_default();
                        tile.collision_shape = None;
                    }
                }
                match Blueprint::save_tileset(&tileset) {
                    Ok(()) => {
                        send_editor_event(EditorEvent::SetTileset(tileset));
                    }
                    Err(err) => error!("Could not update tileset: {:?}", err),
                }
            }
        }
        AdminToSystemEvent::SetTileset(tileset) => match Blueprint::save_tileset(&tileset) {
            Ok(()) => {
                send_editor_event(EditorEvent::SetTileset(tileset));
            }
            Err(err) => error!("Could not update tileset: {:?}", err),
        },
        AdminToSystemEvent::DeleteTileset(tileset) => match Blueprint::delete_tileset(&tileset) {
            Ok(()) => {
                send_editor_event(EditorEvent::DeletedTileset(tileset));
            }
            Err(err) => error!("Could not delete tileset: {:?}", err),
        },
        AdminToSystemEvent::CreateScene(scene) => match Blueprint::create_scene(&scene) {
            Ok(()) => {
                send_editor_event(EditorEvent::CreatedScene(scene));
            }
            Err(err) => error!("Could not create scene: {:?}", err),
        },
        AdminToSystemEvent::UpdateSceneNode(scene_node_update) => match scene_node_update {
            SceneNodeUpdate::UpdateData(resource_path, path, game_node_id, entity_update) => {
                match Blueprint::load_scene(resource_path.clone().into()) {
                    Ok(mut scene) => {
                        match Blueprint::update_node_in_scene(
                            &mut scene,
                            path.clone(),
                            entity_update.clone(),
                        ) {
                            Ok(()) => {
                                send_editor_event(EditorEvent::UpdateScene(
                                    SceneNodeUpdate::UpdateData(
                                        resource_path,
                                        path,
                                        game_node_id,
                                        entity_update,
                                    ),
                                ));
                            }
                            Err(err) => error!("Could not update scene: {:?}", err),
                        }
                    }
                    Err(err) => error!("Could not load scene to update it: {:?}", err),
                }
            }
            SceneNodeUpdate::AddChild(resource_path, path, game_node_id, node) => {
                match Blueprint::load_scene(resource_path.clone().into()) {
                    Ok(mut scene) => {
                        match Blueprint::add_child_in_scene(&mut scene, path.clone(), node.clone())
                        {
                            Ok(()) => {
                                send_editor_event(EditorEvent::UpdateScene(
                                    SceneNodeUpdate::AddChild(
                                        resource_path,
                                        path,
                                        game_node_id,
                                        node,
                                    ),
                                ));
                            }
                            Err(err) => error!("Could not update scene: {:?}", err),
                        }
                    }
                    Err(err) => error!("Could not load scene to update it: {:?}", err),
                }
            }
            SceneNodeUpdate::RemoveChild(resource_path, path, node) => {
                match Blueprint::load_scene(resource_path.clone().into()) {
                    Ok(mut scene) => match Blueprint::remove_child_in_scene(
                        &mut scene,
                        path.clone(),
                        node.clone(),
                    ) {
                        Ok(()) => {
                            debug!("Removed child from scene: {:?}", node);
                            send_editor_event(EditorEvent::UpdateScene(
                                SceneNodeUpdate::RemoveChild(resource_path, path, node),
                            ));
                        }
                        Err(err) => error!("Could not remove scene: {:?}", err),
                    },
                    Err(err) => error!("Could not load scene to remove node: {:?}", err),
                }
            }
        },
        AdminToSystemEvent::DeleteScene(scene) => match Blueprint::delete_scene(&scene) {
            Ok(()) => {
                send_editor_event(EditorEvent::DeletedScene(scene));
            }
            Err(err) => error!("Could not delete scene: {:?}", err),
        },
        AdminToSystemEvent::UpdateModule(module_id, module_update) => {
            debug!("Module update {:?} {:?}", module_map.keys(), module_id);
            if let Some(module) = module_map.get_mut(&module_id) {
                debug!("module found");
                if let Some(new_name) = module_update.name {
                    log_result_error(Blueprint::change_module_name(
                        &mut module.module_blueprint,
                        new_name,
                    ));
                }
                if let Some(main_map) = module_update.main_map {
                    module.module_blueprint.main_map = main_map;
                }
                if let Some(resources) = module_update.resources {
                    match BlueprintService::generate_gid_map(&resources) {
                        Ok(gid_map) => {
                            resource_module.send_resource_event_to(
                                ResourceEvent::UpdateGidMap(gid_map.clone()),
                                module_id.clone(),
                                module.get_active_actor_ids(),
                            );
                            module.module_blueprint.gid_map = gid_map;
                            ConductorModule::update_resource_to_module_map(
                                resource_to_module_map,
                                &module_id,
                                &module.module_blueprint.resources,
                                &resources,
                            );
                            module.update_scripts_from_resources(&resources);
                            module.module_blueprint.resources = resources;
                        }
                        Err(err) => error!("Could not generate gid map! {:?}", err),
                    }
                }
                if let Some(insert_points) = module_update.insert_points {
                    debug!("Updating insert points");
                    if let Ok(mut conductor) = BlueprintService::load_conductor_blueprint() {
                        debug!("Loaded da blueprint");
                        let current_insert_points =
                            Blueprint::io_points_to_hashset(&module.module_blueprint.insert_points);
                        let new_insert_points: HashSet<String> =
                            Blueprint::io_points_to_hashset(&insert_points);
                        let removed_points: HashSet<&String> = current_insert_points
                            .difference(&new_insert_points)
                            .collect();
                        let connections_to_remove: Vec<String> = conductor
                            .module_connection_map
                            .clone()
                            .into_iter()
                            .filter(|(_, (_, insert_point_name))| {
                                removed_points.contains(insert_point_name)
                            })
                            .map(|(exit_slot_name, _)| exit_slot_name)
                            .collect();
                        for connection_to_remove in connections_to_remove {
                            conductor
                                .module_connection_map
                                .remove(&connection_to_remove);
                        }
                        save_and_send_conductor_update(conductor, &mut send_editor_event);
                    }
                    module.module_blueprint.insert_points = insert_points;
                }
                if let Some(exit_points) = module_update.exit_points {
                    if let Ok(mut conductor) = BlueprintService::load_conductor_blueprint() {
                        let current_exit_points =
                            Blueprint::io_points_to_hashset(&module.module_blueprint.exit_points);
                        let new_exit_points: HashSet<String> =
                            Blueprint::io_points_to_hashset(&exit_points);
                        for connection_to_remove in current_exit_points.difference(&new_exit_points)
                        {
                            conductor.module_connection_map.remove(connection_to_remove);
                        }
                        save_and_send_conductor_update(conductor, &mut send_editor_event);
                    }
                    module.module_blueprint.exit_points = exit_points;
                }
                log_result_error(Blueprint::save_module(&module.module_blueprint));
                send_editor_event(EditorEvent::UpdatedModule(
                    module_id,
                    module.module_blueprint.clone(),
                ));
            }
        }
        AdminToSystemEvent::CreateModule(module_name) => {
            match Blueprint::lazy_create_module(&module_name) {
                Ok(module_blueprint) => {
                    if let Some(module_id) = create_game_instance_manager(
                        module_blueprint,
                        module_map,
                        resource_module,
                        resource_to_module_map,
                        module_communication_map,
                    ) {
                        if let Some(module) = module_map.get(&module_id) {
                            send_editor_event(EditorEvent::CreatedModule(
                                module_id,
                                module.module_blueprint.clone(),
                            ));
                        }
                    }
                }
                Err(err) => {
                    debug!("Could not create module: {:?}", err);
                }
            }
        }
        AdminToSystemEvent::CreateScript(script) => match Blueprint::create_script(&script) {
            Ok(()) => {
                send_editor_event(EditorEvent::CreatedScript(script));
            }
            Err(err) => {
                debug!("Could not create script: {:?}", err);
            }
        },
        AdminToSystemEvent::UpdateScript(script) => {
            let mut is_script_compiling = true;
            let script_resource_path =
                format!("{}/{}.script.json", script.resource_path, script.name);
            for module_id in resource_to_module_map
                .entry(script_resource_path.clone())
                .or_default()
                .iter()
            {
                debug!("Updating script in module {:?}", module_id);
                if let Some(module) = module_map.get_mut(module_id) {
                    is_script_compiling &= module.recompile_script(&script, &script_resource_path);
                }
            }

            if is_script_compiling {
                match Blueprint::save_script(&script) {
                    Ok(()) => {
                        send_editor_event(EditorEvent::SetScript(script));
                    }
                    Err(err) => {
                        debug!("Could not create script: {:?}", err);
                    }
                }
            }
        }
        AdminToSystemEvent::DeleteScript(script) => {
            let script_resource_path =
                format!("{}/{}.script.json", script.resource_path, script.name);
            for module_id in resource_to_module_map
                .entry(script_resource_path.clone())
                .or_default()
                .iter()
            {
                if let Some(module) = module_map.get_mut(module_id) {
                    module.remove_script(&script_resource_path);
                }
            }

            match Blueprint::delete_script(&script) {
                Ok(()) => {
                    send_editor_event(EditorEvent::DeletedScript(script));
                }
                Err(err) => {
                    debug!("Could not create script: {:?}", err);
                }
            }
        }
        AdminToSystemEvent::DeleteModule(module_id) => {
            debug!("Deleting module!");
            match remove_game_instance_manager(
                &module_id,
                module_map,
                resource_module,
                module_communication_map,
            ) {
                Ok(()) => {
                    send_editor_event(EditorEvent::DeletedModule(module_id));
                }
                Err(err) => {
                    error!(
                        "Something went wrong while deleting module {}: {:?}",
                        module_id, err
                    );
                }
            }
        }
        AdminToSystemEvent::UpdateInstancedNode(
            module_id,
            game_instance_id,
            world_id,
            entity_update,
        ) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                if let Some(instance) = module.game_instances.get_mut(&game_instance_id) {
                    instance
                        .dynamic_module
                        .apply_admin_entity_update(&world_id, entity_update);
                }
            }
        }
        AdminToSystemEvent::RemoveInstanceNode(module_id, game_instance_id, world_id, entity) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                if let Some(instance) = module.game_instances.get_mut(&game_instance_id) {
                    instance.dynamic_module.remove_entity(&world_id, entity);
                }
            }
        }
        AdminToSystemEvent::AddNodeToInstanceNode(
            module_id,
            game_instance_id,
            world_id,
            parent_entity,
            game_node,
        ) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                if let Some(instance) = module.game_instances.get_mut(&game_instance_id) {
                    instance
                        .dynamic_module
                        .add_entity(&world_id, parent_entity, game_node);
                }
            }
        }
    }
}
