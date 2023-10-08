use crate::core::entity_manager_generation::def::ObjectType;
use crate::core::entity_manager_generation::{
    crate_game_game_object_enum, extract_variants, generate_entity_game_structs,
    generate_entity_imps, generate_entity_manager_impl, generate_entity_manager_struct,
    generate_entity_types, generate_specific_entity_manager_impl, generate_variants, get_imports,
    read_object_types,
};
use crate::resource_module::map::def::TiledMap;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tiled::Tileset;

pub fn generate_entity_manager(
    module_name: &str,
    map_path: &str,
    object_type_paths: &str,
    entity_manager_path: &str,
) {
    let mut object_types: Vec<ObjectType> = read_object_types::read_object_types(object_type_paths);

    let tiled_map = TiledMap::from_xml(PathBuf::from(Path::new("./src")), map_path).unwrap();

    let mut name_to_tileset_map: HashMap<String, (u32, Tileset)> = HashMap::new();
    for tileset_entry in &tiled_map.tilesets {
        if let Some(tileset) = &tileset_entry.tileset {
            name_to_tileset_map.insert(
                tileset.name.clone(),
                (tileset_entry.first_gid, tileset.clone()),
            );
        }
    }

    for object_type in object_types.iter_mut() {
        extract_variants::extract_variants(&mut name_to_tileset_map, object_type);
    }

    let entity_manager_parts: Vec<String> = vec![
        get_imports::get_imports(),
        crate_game_game_object_enum::generate_game_game_object_enum(module_name, &object_types),
        generate_entity_game_structs::generate_entity_game_state_structs(&object_types),
        generate_variants::generate_variants(&object_types),
        generate_entity_types::generate_entity_types(&object_types),
        generate_entity_imps::generate_entity_imps(&object_types),
        generate_entity_manager_struct::generate_entity_manager_struct(module_name, &object_types),
        generate_specific_entity_manager_impl::generate_specific_entity_manager_impl(
            module_name,
            &object_types,
        ),
        generate_entity_manager_impl::generate_entity_manager_impl(module_name, &object_types),
    ];

    let mut file = File::create(entity_manager_path).unwrap();

    file.write_all(entity_manager_parts.join("\n\n").as_bytes())
        .unwrap();
}
