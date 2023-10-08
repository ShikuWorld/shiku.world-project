use crate::core::entity_manager_generation::def::ObjectType;
use codegen::Scope;

pub fn generate_entity_game_state_structs(object_types: &Vec<ObjectType>) -> String {
    let mut structs: Vec<String> = Vec::new();

    for object_type in object_types {
        let mut object_struct_scope = Scope::new();

        let entity_game_state = object_struct_scope
            .new_struct(object_type.name.as_str())
            .vis("pub")
            .derive("Debug");

        for prop in &object_type.props {
            if prop.name.contains("physics")
                || prop.name.contains("render")
                || prop.name.contains("layer_name")
            {
                continue;
            }

            entity_game_state.field(
                format!("pub {}", prop.name).as_str(),
                prop.default.to_string(),
            );
        }

        structs.push(object_struct_scope.to_string());
    }

    structs.join("\n\n")
}
