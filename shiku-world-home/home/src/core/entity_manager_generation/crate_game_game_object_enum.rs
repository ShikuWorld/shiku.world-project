use crate::core::entity_manager_generation::def::ObjectType;
use codegen::Scope;

pub fn generate_game_game_object_enum(module_name: &str, object_types: &Vec<ObjectType>) -> String {
    let mut enum_scope = Scope::new();
    let game_object_enum = enum_scope
        .new_enum(format!("{}GameObject", module_name).as_str())
        .vis("pub");

    game_object_enum
        .derive("Debug")
        .derive("Deserialize")
        .derive("PartialEq")
        .new_variant("Terrain");

    for object_type in object_types {
        game_object_enum.new_variant(object_type.name.clone().as_str());
    }

    enum_scope.to_string()
}
