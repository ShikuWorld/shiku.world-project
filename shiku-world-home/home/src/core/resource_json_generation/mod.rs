use crate::core::get_out_dir;
use crate::resource_module::def::{ResourceConfig, ResourceFile, ResourceKind, TileSetResourceDef};
use crate::resource_module::map::def::TiledMap;
use std::path::PathBuf;

fn get_relative_image_path(path_buf: PathBuf) -> String {
    let path = path_buf.as_path();

    let mut items: Vec<&str> = path
        .iter()
        .rev()
        .map(|a| a.to_str().unwrap())
        .take_while(|a| *a != "shared")
        .collect();

    items = items.into_iter().rev().collect();

    items.join("/")
}

fn get_image_file_name(path_buf: PathBuf) -> String {
    let path = path_buf.as_path();

    let mut items: Vec<&str> = path
        .iter()
        .rev()
        .map(|a| a.to_str().unwrap())
        .take(1)
        .collect();

    items.join("")
}

pub fn generate_resource_map_from_tiled_map(module_name: &str, map_path: &str) -> ResourceFile {
    let tiled_map = TiledMap::from_xml(get_out_dir(), map_path).unwrap();
    let mut resources = Vec::new();

    for tileset in tiled_map.tilesets {
        if let Some(tileset_info) = tileset.tileset {
            let tileset_name = tileset_info.name.clone();
            let tileset_path_buffer = PathBuf::from(tileset.path);

            resources.push(ResourceConfig {
                path: get_relative_image_path(tileset_path_buffer),
                kind: ResourceKind::TileSet(TileSetResourceDef {
                    start_gid: tileset.first_gid.to_string(),
                }),
                meta_name: tileset_name.clone(),
            });

            if let Some(image) = tileset_info.image {
                resources.push(ResourceConfig {
                    path: get_relative_image_path(image.source),
                    kind: ResourceKind::Image,
                    meta_name: format!("{}Image", tileset_name),
                });
            } else {
                for (_id, tile) in tileset_info.tiles() {
                    if let Some(image) = &tile.image {
                        let image_name = get_image_file_name(image.source.clone());

                        resources.push(ResourceConfig {
                            path: get_relative_image_path(image.source.clone()),
                            kind: ResourceKind::Image,
                            meta_name: format!(
                                "{}{}",
                                tileset_name,
                                &image_name[0..image_name.len() - 4]
                            ),
                        });
                    }
                }
            }
        }
    }

    ResourceFile {
        module_name: module_name.to_string(),
        resources,
    }
}
