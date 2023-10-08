use crate::core::entity_manager_generation::def::ObjectType;
use codegen::Scope;
use convert_case::{Case, Casing};

pub fn generate_entity_manager_struct(module_name: &str, object_types: &Vec<ObjectType>) -> String {
    let mut entity_manager_struct_scope = Scope::new();
    let entity_manager_struct = entity_manager_struct_scope
        .new_struct(format!("{}GameEntityManager", module_name).as_str())
        .vis("pub");

    for object_type in object_types {
        entity_manager_struct.field(
            format!("pub {}_map", object_type.name.to_case(Case::Snake)).as_str(),
            format!("HashMap<EntityId, {}Entity>", object_type.name).as_str(),
        );
    }

    entity_manager_struct.field("entity_id_generator", "SnowflakeIdBucket");
    entity_manager_struct.field("pub terrain_map", "HashMap<EntityId, TerrainEntity>");
    entity_manager_struct.field(
        "pub collider_entity_map",
        format!(
            "ColliderEntityMap<{module_name}GameObject>",
            module_name = module_name
        ),
    );
    entity_manager_struct.field("terrain_chunks", "Vec<TerrainChunk>");
    entity_manager_struct.field(
        "guest_id_to_camera_entity_id_map",
        "HashMap<GuestId, EntityId>",
    );
    entity_manager_struct.field("new_show_entities", "Vec<ShowEntity>");
    entity_manager_struct.field("new_remove_entities", "Vec<RemoveEntity>");
    entity_manager_struct.field("pub new_show_effects", "Vec<ShowEffect>");

    entity_manager_struct_scope.to_string()
}
