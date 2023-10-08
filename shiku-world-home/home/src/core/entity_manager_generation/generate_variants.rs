use crate::core::entity_manager_generation::def::ObjectType;

pub fn generate_variants(object_types: &Vec<ObjectType>) -> String {
    let mut structs: Vec<String> = Vec::new();

    for object_type in object_types {
        if object_type.variants.is_empty() {
            continue;
        }

        if let Some(default_variant) = object_type.variants.get("default") {
            let mut variant_struct: Vec<String> = vec![format!(
                "pub struct {object_type_name}Variant {{",
                object_type_name = object_type.name
            )];

            for (name, _gid) in default_variant.gids.iter() {
                variant_struct.push(format!("pub gid_{}: &'static str,", name));
            }

            for (name, _shape) in default_variant.shapes.iter() {
                variant_struct.push(format!("pub shape_{}: PhysicalShape,", name));
            }

            if default_variant.shapes.is_empty() {
                variant_struct.push("pub shape_default: PhysicalShape,".to_string());
            }

            variant_struct.push(
                "pub offset_from_center_x: Real, pub offset_from_center_y: Real,".to_string(),
            );

            variant_struct.push("}".to_string());

            structs.push(variant_struct.join("\n"));

            let variant_impl = format!(
                "impl {object_type_name}Variant {{
                    pub fn get_offset_2d(&self) -> (Real, Real) {{
                        (self.offset_from_center_x, self.offset_from_center_y)
                    }}
                }}",
                object_type_name = object_type.name
            );

            structs.push(variant_impl);
        }

        let mut variants_struct: Vec<String> = vec![format!(
            "pub struct {object_type_name}Variants {{",
            object_type_name = object_type.name
        )];

        let mut add_variants_to_object_type_def: Vec<String> = vec![format!(
            "impl {object_type_name} {{",
            object_type_name = object_type.name
        )];
        add_variants_to_object_type_def.push(format!(
            "pub const VARIANTS: {object_type_name}Variants = {object_type_name}Variants {{",
            object_type_name = object_type.name
        ));

        let mut impl_variants: Vec<String> = vec![format!(
            "impl {object_type_name}Variants {{",
            object_type_name = object_type.name
        )];
        impl_variants.push(format!(
            "pub fn get_variant(&self, variant: &String) -> &{object_type_name}Variant {{",
            object_type_name = object_type.name
        ));
        impl_variants.push("match variant.as_str() {".to_string());

        for (name, variant) in object_type.variants.iter() {
            variants_struct.push(format!("pub {}: {}Variant,", name, object_type.name));
            impl_variants.push(format!("\"{}\" => &self.{},", name, name));
            add_variants_to_object_type_def
                .push(format!("{}: {}Variant {{", name, object_type.name));
            for (name, gid) in variant.gids.iter() {
                add_variants_to_object_type_def.push(format!("gid_{}: \"{}\",", name, gid));
            }
            for (name, shape) in variant.shapes.iter() {
                add_variants_to_object_type_def.push(format!("shape_{}: {},", name, shape));
                if name == &object_type.physics_position {
                    let offset = shape.get_offset_2d();
                    add_variants_to_object_type_def.push(format!(
                        "offset_from_center_x: {:.2}, offset_from_center_y: {:.2},",
                        offset.0, offset.1
                    ));
                }
            }
            if variant.shapes.is_empty() {
                add_variants_to_object_type_def
                    .push("shape_default: PhysicalShape::None,".to_string());
                add_variants_to_object_type_def.push(format!(
                    "offset_from_center_x: {:.2}, offset_from_center_y: {:.2},",
                    0.0, 0.0
                ));
            }
            add_variants_to_object_type_def.push("},".to_string());
        }
        impl_variants.push("_ => &self.default,".to_string());
        impl_variants.push("}".to_string());
        impl_variants.push("}".to_string());
        impl_variants.push("}".to_string());
        variants_struct.push("}".to_string());
        add_variants_to_object_type_def.push("};".to_string());
        add_variants_to_object_type_def.push("}".to_string());

        structs.push(variants_struct.join("\n"));
        structs.push(add_variants_to_object_type_def.join("\n"));
        structs.push(impl_variants.join("\n"));

        if object_type.physics.len() > 1 {
            let mut specific_physics = vec![format!(
                "pub struct Physics{object_type_name} {{",
                object_type_name = object_type.name
            )];
            for (name, physics) in object_type.physics.iter() {
                specific_physics.push(format!("pub {}: Physics{},", name, physics));
            }
            specific_physics.push("}".to_string());
            structs.push(specific_physics.join("\n"));

            let mut collider_handles = Vec::new();
            let mut create_physics = Vec::new();
            let mut remove_physics = Vec::new();
            for (name, physics) in object_type.physics.iter() {
                collider_handles.push(format!("self.{}.collider_handle,", name));
                remove_physics.push(format!("self.{}.remove(physics);", name));
                create_physics.push(format!(
                    "{name}: Physics{physics_class}::create(
                position,
                &build_instructions.shape_{name},
                physics,
            ),",
                    physics_class = physics,
                    name = name
                ));
            }

            let specific_physics_impl = format!(
                "impl Physical for Physics{object_type_name} {{
                type Instruction = {object_type_name}Variant;

                fn position(&self, physics: &RapierSimulation) -> Isometry<Real> {{
                    self.{main_pos_name}.position(physics)
                }}
            
                fn velocity(&self, physics: &RapierSimulation) -> Vector<Real> {{
                    self.{main_pos_name}.velocity(physics)
                }}
            
                fn get_all_collider_handles(&self) -> Vec<ColliderHandle> {{
                    vec![
                        {collider_handles_return}
                    ]
                }}
                
                fn create(
                    position: Vector<Real>,
                    build_instructions: &Self::Instruction,
                    physics: &mut RapierSimulation,
                ) -> Self {{
                    Physics{object_type_name} {{
                        {create_physics_fields}
                    }}
                }}
            
                fn remove(&self, physics: &mut RapierSimulation) {{
                    {remove_physics_fields}
                }}
            }}
                ",
                collider_handles_return = collider_handles.join("\n"),
                create_physics_fields = create_physics.join("\n"),
                remove_physics_fields = remove_physics.join("\n"),
                main_pos_name = object_type.physics_position,
                object_type_name = object_type.name
            );

            structs.push(specific_physics_impl);
        }
    }

    structs.join("\n\n")
}
