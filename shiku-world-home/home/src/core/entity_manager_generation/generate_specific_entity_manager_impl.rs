use crate::core::entity_manager_generation::def::{ObjectType, RenderType};
use convert_case::{Case, Casing};

pub fn generate_specific_entity_manager_impl(
    module_name: &str,
    object_types: &Vec<ObjectType>,
) -> String {
    let mut entity_api_methods = Vec::new();
    let mut entity_maps_creations = Vec::new();

    for object_type in object_types {
        let entity_name_snake_case = object_type.name.to_case(Case::Snake);
        entity_maps_creations.push(format!(
            "            {entity_name_snake_case}_map: HashMap::new(),",
            entity_name_snake_case = entity_name_snake_case
        ));

        if let RenderType::NoRender = object_type.render {
            continue;
        }

        entity_api_methods.push(format!(
            "   pub fn create_{entity_name_snake_case}<F: FnMut(&mut {entity_name}Entity)>(
        &mut self,
        game_state: {entity_name},
        pos: Vector<Real>,
        physics_instructions: &{physics_instructions},
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {{
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = {entity_name}Entity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::{layer_name},
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {{
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), {module_name}GameObject::{entity_name}),
            );
        }}

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.{entity_name_snake_case}_map.insert(entity_id.clone(), entity);
    
        entity_id
    }}
    
    pub fn remove_{entity_name_snake_case}(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<{entity_name}Entity> {{
        if let Some(entity) = self.{entity_name_snake_case}_map.remove(entity_id) {{
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {{
                self.collider_entity_map.remove(&collider_handle);
            }}

            return Some(entity);
        }}

        None
    }}",
            module_name = module_name,
            physics_instructions = if object_type.variants.is_empty() {"PhysicalShape".to_string()} else {format!("{}Variant", object_type.name)},
            entity_name_snake_case=entity_name_snake_case,
            entity_name = object_type.name,
            layer_name = object_type.layer_name
        ));
    }

    format!(
        "impl {module_name}GameEntityManager {{
    pub fn new() -> {module_name}GameEntityManager {{
        {module_name}GameEntityManager {{
            entity_id_generator: SnowflakeIdBucket::new(1, 2),
            collider_entity_map: ColliderEntityMap::new(),
     
            terrain_map: HashMap::new(),
{entity_maps_creations}
            terrain_chunks: Vec::new(),
            guest_id_to_camera_entity_id_map: HashMap::new(),
            new_show_entities: Vec::new(),
            new_remove_entities: Vec::new(),
            new_show_effects: Vec::new(),
        }}
    }}
    {entity_api_methods}
}}",
        module_name = module_name,
        entity_api_methods = entity_api_methods.join("\n\n"),
        entity_maps_creations = entity_maps_creations.join("\n"),
    )
}
