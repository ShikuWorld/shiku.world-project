use log::{debug, error};

use crate::conductor_module::def::{ModuleCommunicationMap, ModuleMap, ResourceToModuleMap};
use crate::core::blueprint::def::{BlueprintError, Module, ModuleId};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::module::ModuleIO;
use crate::core::module_system::game_instance::GameInstanceManager;
use crate::resource_module::def::ResourceModule;

pub fn create_game_instance_manager(
    module_blueprint: Module,
    module_map: &mut ModuleMap,
    resource_module: &mut ResourceModule,
    resource_to_module_map: &mut ResourceToModuleMap,
    module_communication_map: &mut ModuleCommunicationMap,
) -> Option<ModuleId> {
    match GameInstanceManager::new(module_blueprint, resource_module) {
        Ok((game_instance_manager, module_input_sender, module_output_receiver)) => {
            let module_id = game_instance_manager.module_blueprint.id.clone();
            for resource in &game_instance_manager.module_blueprint.resources {
                debug!("Adding resource to resource_to_module_map: {:?}", resource);
                resource_to_module_map
                    .entry(resource.path.clone())
                    .or_default()
                    .insert(module_id.clone());
            }
            module_map.insert(module_id.clone(), game_instance_manager);

            module_communication_map.insert(
                module_id.clone(),
                ModuleIO {
                    receiver: module_output_receiver,
                    sender: module_input_sender,
                },
            );
            Some(module_id)
        }
        Err(err) => {
            error!("Could not create dynamic module: {:?}", err);
            None
        }
    }
}

pub fn remove_game_instance_manager(
    module_id: &ModuleId,
    module_map: &mut ModuleMap,
    resource_module: &mut ResourceModule,
    module_communication_map: &mut ModuleCommunicationMap,
) -> Result<Module, BlueprintError> {
    return if let Some(instance_manager) = module_map.remove(module_id) {
        module_communication_map.remove(module_id);
        resource_module.unregister_resources_for_module(module_id);
        let module_blueprint_name = instance_manager.module_blueprint.name.clone();
        let module_blueprint = instance_manager.module_blueprint;
        Blueprint::delete_module(&module_blueprint_name)?;
        Ok(module_blueprint)
    } else {
        Err(BlueprintError::FileDoesNotExist(
            "Instance Manager not there".into(),
        ))
    };
}
