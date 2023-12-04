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
use crate::core::{safe_unwrap, send_and_log_error_consume};
use crate::resource_module::def::{
    LoadResource, PicUpdateEvent, ResourceBundle, ResourceEvent, ResourceModule,
    ResourceModuleBookKeeping, ResourceModulePicUpdates,
};
use crate::resource_module::errors::{ReadResourceMapError, SendUnloadEventError};

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
            book_keeping: ResourceModuleBookKeeping {
                active_resources: HashMap::new(),
                path_to_module_map: HashMap::new(),
                module_actor_set: HashMap::new(),
                resources: HashMap::new(),
                resource_hash_gen: SnowflakeIdBucket::new(1, 7),
            },
            pic_updates: ResourceModulePicUpdates {
                pic_changed_events_hash: HashSet::new(),
                pic_update_receiver: receiver,
                last_insert: Instant::now(),
            },
            resource_load_events: Vec::new(),
        }
    }

    pub fn receive_all_picture_updates(&mut self) {
        for d in self.pic_updates.pic_update_receiver.drain() {
            self.pic_updates.pic_changed_events_hash.insert(d.path);
            self.pic_updates.last_insert = Instant::now();
        }
    }

    pub fn process_picture_updates(&mut self) {
        if Instant::now()
            .duration_since(self.pic_updates.last_insert)
            .as_millis()
            > 500
        {
            for resource_path in self.pic_updates.pic_changed_events_hash.drain() {
                let mut r = None;
                let mut m = None;
                if let Some(module_id) = self.book_keeping.path_to_module_map.get(&resource_path) {
                    m = Some(module_id.clone());
                    if let Some(map) = self.book_keeping.resources.get(module_id) {
                        if let Some(resource) = map.get(&resource_path) {
                            r = Some(resource.clone());
                        }
                    }
                }
                if let (Some(resource), Some(module_id)) = (r, m) {
                    Self::register_resource_for_module_static(
                        &mut self.book_keeping,
                        &mut self.resource_load_events,
                        module_id,
                        resource,
                    );
                }
            }
        }
    }

    pub fn unregister_resources_for_module(&mut self, module_id: &ModuleId) {
        if let Some(resource_map) = self.book_keeping.resources.remove(module_id) {
            for resource_path in resource_map.keys() {
                self.book_keeping.path_to_module_map.remove(resource_path);
            }
        }
        if let Some(actor_ids) = self.book_keeping.module_actor_set.get(module_id) {
            for actor_id in actor_ids {
                Self::send_unload_event(
                    &mut self.resource_load_events,
                    actor_id,
                    module_id.clone(),
                );
            }
        }
    }

    pub fn unregister_resource_for_module(
        &mut self,
        module_id: &ModuleId,
        resource_path: &ResourcePath,
    ) {
        if let Some(resource_map) = self.book_keeping.resources.get_mut(module_id) {
            self.book_keeping.path_to_module_map.remove(resource_path);
            resource_map.remove(resource_path);
        }
    }

    pub fn register_resource_for_module(&mut self, module_id: ModuleId, resource: LoadResource) {
        Self::register_resource_for_module_static(
            &mut self.book_keeping,
            &mut self.resource_load_events,
            module_id,
            resource,
        );
    }

    fn register_resource_for_module_static(
        book_keeping: &mut ResourceModuleBookKeeping,
        resource_load_events: &mut Vec<(ActorId, ModuleId, ResourceEvent)>,
        module_id: ModuleId,
        resource: LoadResource,
    ) {
        let resource_map = book_keeping.resources.entry(module_id.clone()).or_default();
        let new_resource = LoadResource {
            kind: resource.kind.clone(),
            path: resource.path.clone(),
            cache_hash: book_keeping.resource_hash_gen.get_id().to_string(),
        };
        book_keeping
            .path_to_module_map
            .insert(new_resource.path.clone(), module_id.clone());
        resource_map.insert(new_resource.path.clone(), new_resource.clone());
        let update_name = book_keeping.resource_hash_gen.get_id().to_string();
        println!("hm? {:?}", book_keeping.module_actor_set);
        if let Some(actor_ids) = book_keeping.module_actor_set.get(&module_id) {
            for actor_id in actor_ids {
                Self::send_load_event(
                    resource_load_events,
                    actor_id,
                    module_id.clone(),
                    update_name.clone(),
                    vec![new_resource.clone()],
                );
            }
        }
    }

    pub fn send_load_event(
        resource_load_events: &mut Vec<(ActorId, ModuleId, ResourceEvent)>,
        actor_id: &ActorId,
        module_id: ModuleId,
        name: String,
        assets: Vec<LoadResource>,
    ) {
        resource_load_events.push((
            *actor_id,
            module_id,
            ResourceEvent::LoadResource(ResourceBundle { name, assets }),
        ));
    }

    pub fn send_unload_event(
        resource_load_events: &mut Vec<(ActorId, ModuleId, ResourceEvent)>,
        actor_id: &ActorId,
        module_id: ModuleId,
    ) {
        resource_load_events.push((*actor_id, module_id, ResourceEvent::UnLoadResources));
    }

    pub fn drain_load_events(&mut self) -> std::vec::Drain<'_, (ActorId, ModuleId, ResourceEvent)> {
        self.resource_load_events.drain(..)
    }

    pub fn get_active_resources_for_module(
        &self,
        module_id: &ModuleId,
        guest_id: &ActorId,
    ) -> Result<Vec<LoadResource>, ReadResourceMapError> {
        let active_resources = safe_unwrap(
            self.book_keeping.active_resources.get(guest_id),
            ReadResourceMapError::Get,
        )?;

        let mut resources_out = Vec::new();

        if let Some(true) = active_resources.get(module_id) {
            let current_resources_of_module = safe_unwrap(
                self.book_keeping.resources.get(module_id),
                ReadResourceMapError::Get,
            )?;

            for resource in current_resources_of_module.values() {
                resources_out.push(resource.clone());
            }
        }

        Ok(resources_out)
    }
    pub fn get_resources_for_module(
        &self,
        module_id: &ModuleId,
    ) -> Result<Vec<LoadResource>, ReadResourceMapError> {
        let mut resources_out = Vec::new();

        let resources_of_module = safe_unwrap(
            self.book_keeping.resources.get(module_id),
            ReadResourceMapError::Get,
        )?;

        for resource in resources_of_module.values() {
            resources_out.push(resource.clone());
        }

        Ok(resources_out)
    }

    pub fn activate_module_resource_updates(&mut self, module_id: ModuleId, actor_id: &ActorId) {
        self.book_keeping
            .active_resources
            .entry(*actor_id)
            .or_default()
            .insert(module_id.clone(), true);
        self.book_keeping
            .module_actor_set
            .entry(module_id.clone())
            .or_default()
            .insert(*actor_id);
    }

    pub fn disable_module_resource_updates(
        &mut self,
        module_id: ModuleId,
        actor_id: &ActorId,
    ) -> Result<(), SendUnloadEventError> {
        if self.book_keeping.active_resources.get(actor_id).is_none() {
            return Ok(());
        }

        let active_modules_for_guest_map = safe_unwrap(
            self.book_keeping.active_resources.get_mut(actor_id),
            SendUnloadEventError::NoActiveResourceMapForUser,
        )?;

        active_modules_for_guest_map.remove(&module_id);

        self.book_keeping
            .module_actor_set
            .entry(module_id.clone())
            .or_default()
            .insert(*actor_id);

        Self::send_unload_event(&mut self.resource_load_events, actor_id, module_id);

        Ok(())
    }
}
