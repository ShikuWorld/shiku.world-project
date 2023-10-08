use crate::core::entity_manager_generation::def::ObjectType;

pub fn generate_entity_types(object_types: &Vec<ObjectType>) -> String {
    let mut types: Vec<String> = Vec::new();

    for object_type in object_types {
        let physics_type = if object_type.physics.len() == 1 {
            object_type
                .physics
                .get("default")
                .expect("No default value for physics type!")
                .to_string()
        } else {
            format!("{}", object_type.name)
        };

        types.push(format!(
            "pub type {}Entity = Entity<{}, Physics{}, {}>;",
            object_type.name, object_type.name, physics_type, object_type.render
        ));
    }

    types.join("\n")
}
