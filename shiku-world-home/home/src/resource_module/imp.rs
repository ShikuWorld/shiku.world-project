use std::collections::hash_set::Drain;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use flume::unbounded;
use futures_util::{SinkExt, StreamExt};
use log::error;
use snowflake::SnowflakeIdBucket;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;

use crate::core::blueprint::def::{ModuleId, ResourcePath};
use crate::core::guest::ActorId;
use crate::core::module::{GuestEvent, ModuleInstanceEvent};
use crate::core::module_system::def::WorldId;
use crate::core::module_system::game_instance::GameInstanceId;
use crate::core::{safe_unwrap, send_and_log_error_consume};
use crate::resource_module::def::{
    PicUpdateEvent, Resource, ResourceBundle, ResourceEvent, ResourceModule,
};
use crate::resource_module::errors::{
    ReadResourceMapError, SendLoadEventError, SendUnloadEventError,
};

impl ResourceModule {
    pub async fn new() -> ResourceModule {
        let url = Url::parse("wss://resources.shiku.world/ws").unwrap();
        let (ws_stream, _) = connect_async(url).await.unwrap();
        let (mut write, read) = ws_stream.split();
        let (sender, receiver) = unbounded();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_millis(15000)).await;
                if let Err(err) = write.send(Message::Text("Ping".into())).await {
                    error!("Could not send ping?! {:?}", err);
                }
            }
        });
        tokio::spawn(async move {
            let read_future = read.for_each(|message| async {
                if let Ok(d) = message.unwrap().into_text() {
                    match serde_json::from_str(d.as_str()) {
                        Ok(pic_update) => {
                            send_and_log_error_consume::<PicUpdateEvent>(sender.clone(), pic_update)
                        }
                        Err(err) => error!("{:?}", err),
                    }
                }
            });

            read_future.await;
        });

        ResourceModule {
            active_resources: HashMap::new(),
            resources: HashMap::new(),
            resource_load_events: Vec::new(),
            resource_hash_gen: SnowflakeIdBucket::new(1, 7),
            pic_changed_events_hash: HashSet::new(),
            pic_update_receiver: receiver,
            last_insert: Instant::now(),
        }
    }

    pub fn receive_all_picture_updates(&mut self) {
        for d in self.pic_update_receiver.drain() {
            self.pic_changed_events_hash.insert(d.path);
            self.last_insert = Instant::now();
        }
    }

    pub fn drain_picture_updates(&mut self) -> Option<Drain<'_, String>> {
        if Instant::now().duration_since(self.last_insert).as_millis() > 500 {
            return Some(self.pic_changed_events_hash.drain());
        }
        None
    }

    pub fn unregister_resources_for_module(&mut self, module_id: &ModuleId) {
        self.resources.remove(module_id);
    }

    pub fn unregister_resource_for_module(
        &mut self,
        module_id: &ModuleId,
        resource_path: &ResourcePath,
    ) {
        if let Some(resource_map) = self.resources.get_mut(module_id) {
            resource_map.remove(resource_path);
        }
    }

    pub fn register_resource_for_module(&mut self, module_id: ModuleId, resource: Resource) {
        self.resources.entry(module_id).or_default().insert(
            resource.path.clone(),
            Resource {
                kind: resource.kind,
                path: resource.path,
                cache_hash: self.resource_hash_gen.get_id(),
            },
        );
    }

    pub fn send_load_event(
        &mut self,
        guest_id: &ActorId,
        module_id: &ModuleId,
        instance_id: GameInstanceId,
        world_id: Option<WorldId>,
        name: String,
        assets: Vec<Resource>,
    ) {
        self.resource_load_events.push(GuestEvent {
            guest_id: *guest_id,
            event_type: ModuleInstanceEvent {
                module_id: module_id.clone(),
                instance_id,
                world_id,
                event_type: ResourceEvent::LoadResource(ResourceBundle { name, assets }),
            },
        });
    }

    pub fn send_unload_event(
        &mut self,
        guest_id: ActorId,
        module_id: ModuleId,
        instance_id: GameInstanceId,
        world_id: Option<WorldId>,
    ) {
        self.resource_load_events.push(GuestEvent {
            guest_id,
            event_type: ModuleInstanceEvent {
                module_id,
                instance_id,
                world_id,
                event_type: ResourceEvent::UnLoadResources,
            },
        });
    }

    pub fn drain_load_events(&mut self) -> Vec<GuestEvent<ModuleInstanceEvent<ResourceEvent>>> {
        self.resource_load_events.drain(..).collect()
    }

    pub fn get_active_resources_for_module(
        &self,
        module_id: &ModuleId,
        guest_id: &ActorId,
    ) -> Result<Vec<Resource>, ReadResourceMapError> {
        let active_resources = safe_unwrap(
            self.active_resources.get(guest_id),
            ReadResourceMapError::Get,
        )?;

        let mut resources_out = Vec::new();

        if let Some(true) = active_resources.get(module_id) {
            let current_resources_of_module =
                safe_unwrap(self.resources.get(module_id), ReadResourceMapError::Get)?;

            for resource in current_resources_of_module.values() {
                resources_out.push(resource.clone());
            }
        }

        Ok(resources_out)
    }
    pub fn get_resources_for_module(
        &self,
        module_id: &ModuleId,
    ) -> Result<Vec<Resource>, ReadResourceMapError> {
        let mut resources_out = Vec::new();

        let resources_of_module =
            safe_unwrap(self.resources.get(module_id), ReadResourceMapError::Get)?;

        for resource in resources_of_module.values() {
            resources_out.push(resource.clone());
        }

        Ok(resources_out)
    }

    pub fn activate_module_resource_updates(
        &mut self,
        module_id: ModuleId,
        guest_id: &ActorId,
    ) -> Result<(), SendLoadEventError> {
        self.active_resources
            .entry(*guest_id)
            .or_insert_with(HashMap::new)
            .insert(module_id.clone(), true);

        Ok(())
    }

    pub fn disable_module_resource_updates(
        &mut self,
        module_id: ModuleId,
        instance_id: GameInstanceId,
        guest_id: ActorId,
    ) -> Result<(), SendUnloadEventError> {
        if self.active_resources.get(&guest_id).is_none() {
            return Ok(());
        }

        let active_modules_for_guest_map = safe_unwrap(
            self.active_resources.get_mut(&guest_id),
            SendUnloadEventError::NoActiveResourceMapForUser,
        )?;

        active_modules_for_guest_map.remove(&module_id);

        self.send_unload_event(guest_id, module_id, instance_id, None);

        Ok(())
    }
}
