use crate::core::module::{GuestEvent, ModuleName};
use crate::core::safe_unwrap;
use crate::resource_module::def::{
    GuestId, Resource, ResourceEvent, ResourceFile, ResourceMap, ResourceModule,
};
use crate::resource_module::errors::{
    ReadResourceMapError, ResourceParseError, SendLoadEventError, SendUnloadEventError,
};

use snowflake::SnowflakeIdBucket;
use std::collections::HashMap;

impl ResourceModule {
    pub fn new() -> ResourceModule {
        ResourceModule {
            active_resources: HashMap::new(),
            resources: HashMap::new(),
            resource_load_events: Vec::new(),
            resource_hash_gen: SnowflakeIdBucket::new(1, 2),
        }
    }

    pub fn register_resources_for_module(
        &mut self,
        module_name: ModuleName,
        _resource_base_path: String,
        mut resource_file: ResourceFile,
        manual_config_option: Option<String>,
    ) -> Result<(), ResourceParseError> {
        //self.watch_path_for_changes(resource_base_path);
        if let Some(manual_config) = manual_config_option {
            let mut manual_config = ResourceModule::parse_resource_config(manual_config)?;
            resource_file.resources.append(&mut manual_config.resources);
        }

        let module_resources_map = self
            .resources
            .entry(module_name)
            .or_insert_with(|| HashMap::new());

        for resource in resource_file.resources {
            module_resources_map.insert(
                resource.meta_name.clone(),
                Resource {
                    meta_name: resource.meta_name,
                    kind: resource.kind,
                    path: resource.path,
                    cache_hash: self.resource_hash_gen.get_id(),
                },
            );
        }

        Ok(())
    }

    pub fn parse_resource_config(
        resource_config: String,
    ) -> Result<ResourceFile, ResourceParseError> {
        let resources: ResourceFile = serde_json::from_str(&resource_config)?;

        Ok(resources)
    }

    /*pub fn watch_path_for_changes(&mut self, path: String) {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch("/home/test/notify", RecursiveMode::Recursive)
            .unwrap();

        loop {
            match rx.recv() {
                Ok(event) => println!("{:?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }*/

    pub fn send_load_event(&mut self, guest_id: GuestId) -> Result<(), SendLoadEventError> {
        self.resource_load_events.push(GuestEvent {
            guest_id,
            event_type: ResourceEvent::LoadResource(self.read_active_resource_map(guest_id)?),
        });

        Ok(())
    }

    pub fn send_unload_event(&mut self, guest_id: GuestId, module_name: &ModuleName) {
        self.resource_load_events.push(GuestEvent {
            guest_id,
            event_type: ResourceEvent::UnLoadResource(module_name.clone()),
        });
    }

    pub fn drain_load_events(&mut self) -> Vec<GuestEvent<ResourceEvent>> {
        self.resource_load_events.drain(..).collect()
    }

    pub fn read_active_resource_map(
        &self,
        guest_id: GuestId,
    ) -> Result<ResourceMap, ReadResourceMapError> {
        let mut resource_map: HashMap<ModuleName, Vec<Resource>> = HashMap::new();

        let active_resources = safe_unwrap(
            self.active_resources.get(&guest_id),
            ReadResourceMapError::Get,
        )?;

        for (module_name, _bool) in active_resources {
            let resources_out = resource_map
                .entry(module_name.clone())
                .or_insert_with(|| Vec::new());

            let current_resources_of_module =
                safe_unwrap(self.resources.get(module_name), ReadResourceMapError::Get)?;

            for (_resource_meta_name, resource) in current_resources_of_module {
                resources_out.push(resource.clone());
            }
        }

        Ok(ResourceMap(resource_map))
    }

    pub fn activate_resources_for_guest(
        &mut self,
        module_name: ModuleName,
        guest_id: GuestId,
    ) -> Result<(), SendLoadEventError> {
        self.active_resources
            .entry(guest_id.clone())
            .or_insert_with(|| HashMap::new())
            .insert(module_name, true);

        self.send_load_event(guest_id)?;

        Ok(())
    }

    pub fn disable_resources_for_guest(
        &mut self,
        module_name: ModuleName,
        guest_id: GuestId,
    ) -> Result<(), SendUnloadEventError> {
        if let None = self.active_resources.get(&guest_id) {
            return Ok(());
        }

        let active_modules_for_guest_map = safe_unwrap(
            self.active_resources.get_mut(&guest_id),
            SendUnloadEventError::NoActiveResourceMapForUser,
        )?;

        active_modules_for_guest_map.remove(&module_name);

        self.send_unload_event(guest_id, &module_name);

        Ok(())
    }
}
