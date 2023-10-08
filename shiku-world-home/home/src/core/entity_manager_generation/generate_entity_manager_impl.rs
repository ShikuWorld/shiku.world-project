use crate::core::entity_manager_generation::def::{ObjectType, PropertyKind, RenderType};
use convert_case::{Case, Casing};

pub fn generate_entity_manager_impl(module_name: &str, object_types: &Vec<ObjectType>) -> String {
    let mut create_initial_objects = Vec::new();
    let mut assign_references = Vec::new();
    let mut position_updates = Vec::new();
    let mut all_show_entities = Vec::new();
    let mut all_update_entities = Vec::new();
    let mut all_position_updates = Vec::new();

    for object_type in object_types {
        let object_name_snake_case = object_type.name.to_case(Case::Snake);
        create_initial_objects.push(format!(
            "                     {module_name}GameObject::{object_name} => {{
                        let entity = {object_name}Entity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {{
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), {module_name}GameObject::{object_name}),
                            );
                        }}

                        self.{object_name_snake_case}_map.insert(entity_id, entity);
                    }}",
            module_name = module_name,
            object_name = object_type.name,
            object_name_snake_case = object_name_snake_case
        ));

        for prop in &object_type.props {
            if let PropertyKind::Object(_) = &prop.default {
                assign_references.push(format!(
                    "       for entity in self.{object_name_snake_case}_map.values_mut() {{
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.{prop_name}) {{
                entity.game_state.{prop_name} = entity_id.clone();
            }}
        }}",
                    object_name_snake_case = object_name_snake_case,
                    prop_name = prop.name
                ));
            }
        }

        position_updates.push(format!(
            "for entity in self.{object_name_snake_case}_map.values_mut() {{
            entity.update_isometry(physics);
        }}",
            object_name_snake_case = object_name_snake_case
        ));

        if RenderType::NoRender != object_type.render {
            all_show_entities.push(format!(
                "        for entity in self.{object_name_snake_case}_map.values() {{
            show_entities.push(entity.show_entity());
        }}",
                object_name_snake_case = object_name_snake_case
            ));

            all_update_entities.push(format!(
                "        for entity in self.{object_name_snake_case}_map.values_mut() {{
            if entity.is_render_dirty {{
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }}
        }}",
                object_name_snake_case = object_name_snake_case
            ));

            all_position_updates.push(format!(
                "        for entity in self.{object_name_snake_case}_map.values_mut() {{
            if entity.is_position_dirty {{
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }}
        }}",
                object_name_snake_case = object_name_snake_case
            ));
        }
    }

    format!(
        "impl EntityManager for {module_name}GameEntityManager {{
    fn create_initial(&mut self, map: &TiledMap, physics: &mut RapierSimulation) {{
        for layer in &map.layers {{
            self.terrain_chunks.extend(layer.terrain_chunks.clone());

            if let LayerName::Terrain = &layer.name {{
                let chunks = condense_terrain_from_tiles(layer);
                for chunk in chunks {{
                    let (body_handle, collider_handle) =
                        TerrainEntity::create_terrain_collider(&chunk, physics);

                    let terrain_entity = TerrainEntity::new_entity(
                        self.entity_id_generator.get_id().to_string(),
                        Isometry::new(Vector::new(0.0, 0.0), 0.0),
                        body_handle,
                        collider_handle,
                    );
                    self.collider_entity_map.insert(
                        collider_handle,
                        (terrain_entity.id.clone(), {module_name}GameObject::Terrain),
                    );
                    self.terrain_map
                        .insert(terrain_entity.id.clone(), terrain_entity);
                }}
            }}
        }}

        let mut reference_map = HashMap::<ObjectId, EntityId>::new();

        for group in &map.object_groups {{
            for object in &group.objects {{
                if let Ok(kind) = serde_json::from_str(object.kind.as_str()) {{
                    let entity_id = self.entity_id_generator.get_id().to_string();
                    reference_map.insert(object.id.clone(), entity_id.clone());

                    match kind {{
{create_initial_objects}
                        kind => {{
                            debug!(\"Not generated right now {{:?}}\", kind);
                        }}
                    }}
                }}
            }}
        }}

{assign_references}
    }}

    fn update_entity_positions(&mut self, physics: &mut RapierSimulation) {{
{position_updates}
    }}

    fn set_camera_entity_for_guest(&mut self, guest_id: GuestId, entity_id: EntityId) {{
        self.guest_id_to_camera_entity_id_map
            .insert(guest_id, entity_id);
    }}

    fn get_current_camera_entity_for_guest(&self, guest_id: &GuestId) -> EntityId {{
        self.guest_id_to_camera_entity_id_map
            .get(guest_id)
            .unwrap_or(&String::new())
            .clone()
    }}

    fn get_all_terrain_chunks(&mut self) -> Vec<TerrainChunk> {{
        self.terrain_chunks.clone()
    }}

    fn get_all_show_entities(&mut self) -> Vec<ShowEntity> {{
        let mut show_entities = Vec::new();
{all_show_entities}
        show_entities
    }}

    fn get_all_entity_updates(&mut self) -> Vec<UpdateEntity> {{
        let mut entity_updates = Vec::new();
{all_update_entities}
        entity_updates
    }}

    fn get_all_entity_position_updates(&mut self) -> Vec<(EntityId, Real, Real, Real)> {{
        let mut position_updates = Vec::new();
{all_position_updates}
        position_updates
    }}

    fn drain_new_show_effects(&mut self) -> Vec<ShowEffect> {{
        self.new_show_effects.drain(..).collect()
    }}

    fn drain_new_show_entities(&mut self) -> Vec<ShowEntity> {{
        self.new_show_entities.drain(..).collect()
    }}

    fn drain_new_remove_entities(&mut self) -> Vec<RemoveEntity> {{
        self.new_remove_entities.drain(..).collect()
    }}
}}",
        create_initial_objects = create_initial_objects.join("\n"),
        assign_references = assign_references.join("\n"),
        position_updates = position_updates.join("\n"),
        module_name = module_name,
        all_show_entities = all_show_entities.join("\n"),
        all_update_entities = all_update_entities.join("\n"),
        all_position_updates = all_position_updates.join("\n")
    )
}
