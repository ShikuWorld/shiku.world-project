use crate::core::entity_manager_generation::def::{ObjectType, RenderType};
use convert_case::{Case, Casing};

pub fn generate_entity_imps(object_types: &[ObjectType]) -> String {
    let mut imps: Vec<String> = Vec::new();

    for object_type in object_types {
        let mut entity_imp = Vec::new();

        entity_imp.push(format!("impl {}Entity {{", object_type.name));

        let mut game_state_from_object = Vec::new();

        for object_type_prop in &object_type.props {
            if object_type_prop.name.contains("physics")
                || object_type_prop.name.contains("render")
                || object_type_prop.name.contains("layer_name")
            {
                continue;
            }

            game_state_from_object.push(format!(
                "   {}: obj.get_custom_prop_{}(\"{}\"),",
                object_type_prop.name,
                &object_type_prop.default.to_string().to_case(Case::Snake),
                object_type_prop.name
            ));
        }

        entity_imp.push(format!(
            "    pub fn game_state_from_general_object(obj: &GeneralObject) -> {object_type_name} {{
        {object_type_name} {{
{game_state_from_object}
        }}
    }}", game_state_from_object = game_state_from_object.join("\n"),
            object_type_name = object_type.name
        ));

        insert_new_fn(&mut entity_imp, object_type);

        imps.push(entity_imp.join("\n\n"));
    }

    imps.join("\n\n")
}

fn insert_new_fn(entity_imp: &mut Vec<String>, object_type: &ObjectType) {
    let object_type_name = object_type.name.clone();
    let physics_type_class = if object_type.physics.len() == 1 {
        object_type
            .physics
            .get("default")
            .expect("No default value for physics type!")
            .to_string()
    } else {
        object_type_name.to_string()
    };

    let create_physics_code = format!(
        "let physics_body = Physics{physics_type_class}::create(
                pos,
                &physics_instructions{use_default_shape},
                physics,
            );",
        use_default_shape = if object_type.variants.is_empty() || object_type.physics.len() > 1 {
            ""
        } else {
            ".shape_default"
        },
        physics_type_class = physics_type_class
    );

    let render_object: &str = match object_type.render {
        RenderType::StaticImage => "StaticImage { width: None, height: None, tiled: false, layer, graphic_id, blending_mode: None, scale: (1.0, 1.0), offset_2d: physics_instructions.get_offset_2d() }",
        RenderType::NoRender => "NoRender {}",
        RenderType::RenderTypeTimer => {
            "RenderTypeTimer::from_general_object(&general_object, layer)"
        }
        RenderType::RenderTypeText => {
            "RenderTypeText::from_general_object(&general_object, layer)"
        }
    };

    entity_imp.push(format!(
        "    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &{physics_instructions_type},
        game_state: {object_type_name},
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> {object_type_name}Entity {{
{create_physics_code}

        Entity {{
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: {render_object},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }}
    }}",
        object_type_name = object_type_name,
        physics_instructions_type = if object_type.variants.is_empty() {
            "PhysicalShape".to_string()
        } else {
            format!("{}Variant", object_type_name)
        },
        create_physics_code = create_physics_code,
        render_object = render_object
    ));

    let get_physics_instructions_from_obj = if object_type.variants.is_empty() {
        "&physical_shape_from_general_obj(general_object)".to_string()
    } else {
        format!(
            "{}::VARIANTS.get_variant(&general_object.get_custom_prop_string(\"variant\"))",
            object_type_name,
        )
    };

    entity_imp.push(format!(
        "    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> {object_type_name}Entity {{
        let physics_instructions = {get_physics_instructions_from_obj};

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            {object_type_name}Entity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }}",
        object_type_name = object_type_name,
        get_physics_instructions_from_obj = get_physics_instructions_from_obj
    ));

    entity_imp.push("}".to_string());
}
