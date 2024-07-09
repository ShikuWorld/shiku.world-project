use crate::core::entity::physics::{PhysicalShape, ShapePoint, ShapeRect};
use crate::core::entity_manager_generation::def::{ObjectType, ObjectVariant};
use rapier2d::math::Real;
use std::collections::HashMap;
use tiled::{ObjectShape, PropertyValue, Tileset};

pub fn extract_variants(
    name_to_tileset_map: &mut HashMap<String, (u32, Tileset)>,
    object_type: &mut ObjectType,
) {
    if let Some((default_start_gid, tileset)) = name_to_tileset_map.get(&object_type.name) {
        if let Some(default_variant) = get_default_variant(tileset, default_start_gid) {
            let default_variant_start_id = default_variant.start_id;
            let default_variant_gids = default_variant.gids.clone();

            object_type
                .variants
                .insert("default".to_string(), default_variant);

            insert_variants_into_object_type(
                tileset,
                object_type,
                &default_variant_gids,
                |local_tile_id, variant| {
                    local_tile_id + (variant.start_id - default_variant_start_id)
                },
            );

            let object_type_name = object_type.name.clone();

            for (variant_start_gid, tileset) in
                name_to_tileset_map.values().filter(|(_, tileset)| {
                    let possible_variants: Vec<&str> = tileset.name.split('_').collect();
                    if possible_variants.len() == 2 {
                        return possible_variants
                            .get(0)
                            .unwrap()
                            .contains(&object_type_name);
                    }
                    false
                })
            {
                insert_variants_into_object_type(
                    tileset,
                    object_type,
                    &default_variant_gids,
                    |local_tile_id, _variant| {
                        ((*local_tile_id as i32)
                            + (*variant_start_gid as i32 - *default_start_gid as i32))
                            as u32
                    },
                );
            }
        }
    }
}

fn get_default_variant(tileset: &Tileset, gid: &u32) -> Option<ObjectVariant> {
    let mut default_variant_exists = false;
    let mut default_variant = ObjectVariant {
        start_id: 0,
        gids: HashMap::new(),
        shapes: HashMap::new(),
        offset_2d: (0.0, 0.0),
        tile_size: (0.0, 0.0),
    };

    for (id, tile) in tileset.tiles() {
        if let Some(PropertyValue::StringValue(variant_name)) = tile.properties.get("variant") {
            if variant_name == "default" {
                default_variant.start_id = id;
                default_variant_exists = true;
            }
        }

        if let Some(PropertyValue::StringValue(tile_name)) = tile.properties.get("tile_name") {
            {
                default_variant.gids.insert(tile_name.clone(), id + gid);
            }
        }
    }

    if default_variant_exists {
        Some(default_variant)
    } else {
        None
    }
}

fn insert_variants_into_object_type<F>(
    tileset: &Tileset,
    object_type: &mut ObjectType,
    default_variant_gid_map: &HashMap<String, u32>,
    mut calculate_variant_tile_name_gid: F,
) where
    F: FnMut(&u32, &ObjectVariant) -> u32,
{
    for (id, tile) in tileset.tiles() {
        if let Some(PropertyValue::StringValue(variant_name)) = tile.properties.get("variant") {
            if variant_name == "default" {
                continue;
            }

            let mut variant = ObjectVariant {
                start_id: id,
                gids: HashMap::new(),
                shapes: HashMap::new(),
                offset_2d: (0.0, 0.0),
                tile_size: (0.0, 0.0),
            };

            for (name, id) in default_variant_gid_map.iter() {
                variant
                    .gids
                    .insert(name.clone(), calculate_variant_tile_name_gid(id, &variant));
            }

            object_type
                .variants
                .insert(variant_name.to_string(), variant);
        }
    }

    for (_id, tile) in tileset.tiles() {
        let tile_width = if let Some(image) = &tile.image {
            image.width as Real
        } else {
            tileset.tile_width as Real
        };
        let tile_height = if let Some(image) = &tile.image {
            image.height as Real
        } else {
            tileset.tile_height as Real
        };

        if let Some(PropertyValue::StringValue(variant_name)) = tile.properties.get("variant") {
            if let Some(variant) = object_type.variants.get_mut(variant_name) {
                variant.tile_size = (tile_width, tile_height);

                if let Some(collision) = &tile.collision {
                    for collision_obj in collision.object_data() {
                        match collision_obj.shape {
                            ObjectShape::Rect { width, height } => {
                                let shape = ShapeRect {
                                    offset_from_center_x: (width / 2.0) - (tile_width / 2.0)
                                        + collision_obj.x,
                                    offset_from_center_y: (height / 2.0) - (tile_height / 2.0)
                                        + collision_obj.y,
                                    width,
                                    height,
                                };
                                if collision_obj.name == object_type.physics_position {
                                    variant.offset_2d =
                                        (shape.offset_from_center_x, shape.offset_from_center_y);
                                }
                                variant.shapes.insert(
                                    collision_obj.name.clone(),
                                    PhysicalShape::ShapeRect(shape),
                                );
                            }
                            ObjectShape::Point(_x, _y) => {
                                let shape = ShapePoint {
                                    offset_from_center_x: (tile_width / 2.0) + collision_obj.x,
                                    offset_from_center_y: (tile_height / 2.0) + collision_obj.y,
                                };
                                if collision_obj.name == object_type.physics_position {
                                    variant.offset_2d =
                                        (shape.offset_from_center_x, shape.offset_from_center_y);
                                }
                                variant.shapes.insert(
                                    collision_obj.name.clone(),
                                    PhysicalShape::ShapePoint(shape),
                                );
                            }
                            _ => {
                                panic!("Collision object for \"{}\" detected that is not a Rectangle or a Point. Other types are not supported atm.", object_type.name)
                            }
                        }
                    }
                }
            }
        }
    }
}
