use std::collections::HashSet;
use std::path::PathBuf;

use flume::Sender;
use log::{debug, error};
use uuid::Uuid;

use crate::conductor_module::blueprint_helper::save_and_send_conductor_update;
use crate::conductor_module::def::{ModuleCommunicationMap, ModuleMap};
use crate::conductor_module::game_instances::{
    create_game_instance_manager, remove_game_instance_manager,
};
use crate::core::blueprint::def::{
    BlueprintResource, BlueprintService, ResourceKind, ResourceLoaded,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::guest::{ActorId, Admin};
use crate::core::module::{AdminToSystemEvent, CommunicationEvent, EditorEvent};
use crate::core::{log_result_error, send_and_log_error};
use crate::resource_module::def::{ResourceBundle, ResourceModule};
use crate::webserver_module::def::WebServerModule;

pub async fn handle_admin_to_system_event(
    module_communication_map: &mut ModuleCommunicationMap,
    web_server_module: &mut WebServerModule,
    resource_module: &mut ResourceModule,
    module_map: &mut ModuleMap,
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
        AdminToSystemEvent::InitialResourcesLoaded(module_id) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                for instance in module.game_instances.values_mut() {
                    if let Some(module_admin) = instance.dynamic_module.admins.get_mut(&admin.id) {
                        module_admin.resources_loaded = true;
                    }
                }
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
                                send_communication_event(CommunicationEvent::PrepareGame(
                                    module_id,
                                    game_instance_id,
                                    Some(world_id),
                                    ResourceBundle {
                                        name: "Default".into(),
                                        assets,
                                    },
                                ))
                            }
                            Err(err) => {
                                error!("Could not send prepare game, no resources?! {:?}", err)
                            }
                        }
                    }
                    Err(err) => error!("Could not get admin into instance/map {:?}", err),
                }
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
            let map_path = format!("{}/{}", map_update.resource_path, map_update.name);
            match Blueprint::load_map(PathBuf::from(map_path)) {
                Ok(mut map) => {
                    if let Some(entities) = map_update.entities.clone() {
                        map.entities = entities;
                    }
                    if let Some(joints) = map_update.joints.clone() {
                        map.joints = joints;
                    }
                    if let Some((layer, n, chunk)) = map_update.chunk.clone() {
                        if let Some(chunks) = map.terrain.get_mut(&layer) {
                            if chunks.get(n).is_some() {
                                chunks[n] = chunk;
                            }
                        }
                    }
                    match Blueprint::save_map(&map) {
                        Ok(()) => {
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

                        match BlueprintService::save_module(&module.module_blueprint) {
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
                        match BlueprintService::save_module(&module.module_blueprint) {
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
                ResourceLoaded::Map(map) => {
                    send_editor_event(EditorEvent::SetMap(map));
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
            match BlueprintService::get_all_modules() {
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
        AdminToSystemEvent::UpdateModule(module_id, module_update) => {
            if let Some(module) = module_map.get_mut(&module_id) {
                if let Some(new_name) = module_update.name {
                    log_result_error(BlueprintService::change_module_name(
                        &mut module.module_blueprint,
                        new_name,
                    ));
                }
                if let Some(resources) = module_update.resources {
                    module.module_blueprint.resources = resources;
                }
                if let Some(insert_points) = module_update.insert_points {
                    if let Ok(mut conductor) = BlueprintService::load_conductor_blueprint() {
                        let current_insert_points =
                            Blueprint::io_points_to_hashset(&module.module_blueprint.insert_points);
                        let new_insert_points: HashSet<String> =
                            Blueprint::io_points_to_hashset(&insert_points);
                        let removed_points: HashSet<&String> = current_insert_points
                            .difference(&new_insert_points)
                            .collect();
                        debug!("removed_points {:?}", removed_points);
                        let connections_to_remove: Vec<String> = conductor
                            .module_connection_map
                            .clone()
                            .into_iter()
                            .filter(|(_, (_, insert_point_name))| {
                                removed_points.contains(insert_point_name)
                            })
                            .map(|(exit_slot_name, _)| exit_slot_name)
                            .collect();
                        debug!("connections_to_remove {:?}", connections_to_remove);
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
                log_result_error(BlueprintService::save_module(&module.module_blueprint));
                send_editor_event(EditorEvent::UpdatedModule(
                    module_id,
                    module.module_blueprint.clone(),
                ));
            }
        }
        AdminToSystemEvent::CreateModule(module_name) => {
            if BlueprintService::module_exists(&module_name) {
                error!("Module already existed!");
                return;
            }
            if let Some(module_id) = create_game_instance_manager(
                module_name,
                module_map,
                resource_module,
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
    }
}
