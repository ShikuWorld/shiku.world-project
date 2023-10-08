use crate::resource_module::errors::ResourceParseError;

use crate::core::tween::{Tween, TweenProp};
use crate::resource_module::map::def::{
    CustomPropType, GeneralObject, Layer, ObjectGroup, ObjectText, TerrainChunk, TiledMap,
    TilesetEntry,
};
use anyhow::Context;
use log::{debug, error};
use rand::{thread_rng, Rng};
use rapier2d::prelude::{Real, Vector};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use tiled::Loader;
use xml::reader::XmlEvent;
use xml::EventReader;

impl GeneralObject {
    pub fn get_random_point(general_object_option: &Option<GeneralObject>) -> Vector<Real> {
        if let Some(general_object) = general_object_option {
            let mut rng = thread_rng();
            let random_x: Real = rng.gen();
            let random_y: Real = rng.gen();
            return Vector::new(
                general_object.x + general_object.width * random_x,
                general_object.y + general_object.height * random_y,
            );
        }

        Vector::new(0.0, 0.0)
    }
}

impl TiledMap {
    pub fn from_xml(base_path: PathBuf, file_path: &str) -> Result<TiledMap, ResourceParseError> {
        let file = File::open(base_path.join(file_path))?;
        let mut file_base_path = base_path.join(file_path);
        file_base_path.pop();

        let mut tiled_map = TiledMap {
            height: 0,
            width: 0,
            tile_height: 0,
            tile_width: 0,
            tilesets: vec![],
            object_groups: vec![],
            layers: vec![],
        };

        let parser = EventReader::new(BufReader::new(file));

        let mut current_tileset_entry = TilesetEntry::new();
        let mut current_layer = Layer::new();
        let mut current_object_group = ObjectGroup::new();
        let mut current_chunk = TerrainChunk::new();
        let mut current_object_attributes = GeneralObject::new();
        let mut properties_depth = 0;
        let mut current_tween = Tween::new();
        let mut current_prop_propertytype = "".to_string();
        let mut current_prop_name = "".to_string();
        let mut current_text = None;

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => match name.local_name.as_str() {
                    "properties" => {
                        properties_depth += 1;
                    }
                    "map" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "height" => tiled_map.height = attr.value.parse()?,
                                "width" => tiled_map.width = attr.value.parse()?,
                                "tileheight" => tiled_map.tile_height = attr.value.parse()?,
                                "tilewidth" => tiled_map.tile_width = attr.value.parse()?,
                                _ => (),
                            }
                        }
                    }
                    "tileset" => {
                        for attr in attributes {
                            if attr.name.local_name.as_str() == "firstgid" {
                                current_tileset_entry.first_gid = attr.value.parse()?
                            }
                            if attr.name.local_name.as_str() == "source" {
                                let mut loader = Loader::new();
                                current_tileset_entry.path = attr.value.clone();
                                match loader.load_tsx_tileset(file_base_path.join(attr.value)) {
                                    Ok(tileset) => current_tileset_entry.tileset = Some(tileset),
                                    Err(err) => error!("{:?}", err),
                                }
                            }
                        }
                    }
                    "layer" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "id" => current_layer.id = attr.value,
                                "parallaxx" => current_layer.parallax.0 = attr.value.parse()?,
                                "parallaxy" => current_layer.parallax.1 = attr.value.parse()?,
                                "name" => {
                                    let wrapped_value = format!("\"{}\"", attr.value);
                                    current_layer.name = serde_json::from_str(
                                        wrapped_value.as_str(),
                                    )
                                    .context(format!(
                                        "Could not parse name for layer {}.",
                                        attr.value
                                    ))?
                                }
                                _ => (),
                            }
                        }
                    }
                    "data" => {
                        for attr in attributes {
                            if attr.name.local_name.as_str() == "encoding" {
                                current_layer.encoding = attr.value
                            }
                        }
                    }
                    "objectgroup" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "id" => current_object_group.id = attr.value,
                                "name" => {
                                    let wrapped_value = format!("\"{}\"", attr.value);
                                    current_object_group.layer_name = serde_json::from_str(
                                        wrapped_value.as_str(),
                                    )
                                    .context(format!(
                                        "Could not parse name for object group {}.",
                                        attr.value
                                    ))?
                                }
                                _ => (),
                            }
                        }
                    }
                    "object" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "id" => current_object_attributes.id = attr.value,
                                "x" => current_object_attributes.x = attr.value.parse()?,
                                "y" => current_object_attributes.y = attr.value.parse()?,
                                "width" => current_object_attributes.width = attr.value.parse()?,
                                "height" => {
                                    current_object_attributes.height = attr.value.parse()?
                                }
                                "gid" => {
                                    current_object_attributes.graphic_id = attr.value;
                                }
                                "class" => {
                                    let wrapped_value = format!("\"{}\"", attr.value);
                                    current_object_attributes.kind = wrapped_value;
                                }
                                _ => (),
                            }
                        }
                        current_object_attributes.y -= tiled_map.tile_height as Real;
                    }
                    "property" => {
                        let mut prop_name: String = "".to_string();
                        let mut prop_value: String = "".to_string();
                        let mut prop_type: String = "".to_string();
                        let mut prop_propertytype: String = "".to_string();

                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "value" => prop_value = attr.value,
                                "name" => prop_name = attr.value,
                                "type" => prop_type = attr.value,
                                "propertytype" => prop_propertytype = attr.value,
                                _ => (),
                            }
                        }

                        if properties_depth == 2 && current_prop_propertytype == "Tween" {
                            match prop_name.as_str() {
                                "add_value" => current_tween.add_value = prop_value.parse()?,
                                "time" => current_tween.set_time(prop_value.parse()?),
                                "repeat" => current_tween.repeat = prop_value == "true",
                                "property" => {
                                    if prop_value == "PositionX" {
                                        current_tween.property = TweenProp::PositionX;
                                        current_tween.initial_value = current_object_attributes.x;
                                    }
                                }
                                _ => (),
                            }
                            continue;
                        }

                        current_prop_name = prop_name.clone();

                        match prop_type.as_str() {
                            "float" => {
                                current_object_attributes
                                    .custom_props
                                    .insert(prop_name, CustomPropType::Float(prop_value.parse()?));
                            }
                            "int" => {
                                current_object_attributes
                                    .custom_props
                                    .insert(prop_name, CustomPropType::Int(prop_value.parse()?));
                            }
                            "bool" => {
                                current_object_attributes.custom_props.insert(
                                    prop_name,
                                    CustomPropType::Boolean(prop_value.parse()?),
                                );
                            }
                            "string" | "" => {
                                current_object_attributes
                                    .custom_props
                                    .insert(prop_name, CustomPropType::String(prop_value));
                            }
                            "object" => {
                                current_object_attributes
                                    .custom_props
                                    .insert(prop_name, CustomPropType::Object(prop_value));
                            }
                            "class" => {
                                current_prop_propertytype = prop_propertytype;
                                if current_prop_propertytype == "Tween" {
                                    current_tween = Tween::new();
                                    current_tween.initial_value = current_object_attributes.y;
                                }
                            }
                            _ => {
                                error!(
                                    "Undefined type '{:?}' for property '{:?}'",
                                    prop_type, prop_name
                                );
                            }
                        }
                    }
                    "chunk" => {
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "x" => current_chunk.x = attr.value.parse()?,
                                "y" => current_chunk.y = attr.value.parse()?,
                                "width" => current_chunk.width = attr.value.parse()?,
                                "height" => current_chunk.height = attr.value.parse()?,
                                _ => (),
                            }
                        }
                        current_chunk.layer = current_layer.name.clone();
                    }
                    "text" => {
                        let mut text = ObjectText::new();

                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "fontfamily" => text.font_family = attr.value,
                                "pixelsize" => text.pixel_size = attr.value,
                                "color" => text.color = attr.value,
                                _ => (),
                            }
                        }

                        current_text = Some(text);
                    }
                    _ => {
                        debug!("Opening {:?}", name);
                    }
                },
                Ok(XmlEvent::EndElement { name }) => match name.local_name.as_str() {
                    "properties" => {
                        if properties_depth == 2 && current_prop_propertytype == "Tween" {
                            current_object_attributes.custom_props.insert(
                                current_prop_name.clone(),
                                CustomPropType::Tween(current_tween.clone()),
                            );
                        }
                        properties_depth -= 1;
                    }
                    "text" => {
                        current_object_attributes.text = current_text;

                        current_text = None;
                    }
                    "tileset" => {
                        tiled_map.tilesets.push(current_tileset_entry.clone());
                        current_tileset_entry = TilesetEntry::new()
                    }
                    "objectgroup" => {
                        tiled_map.object_groups.push(current_object_group.clone());
                        current_object_group = ObjectGroup::new();
                    }
                    "layer" => {
                        tiled_map.layers.push(current_layer.clone());
                        current_layer = Layer::new();
                    }
                    "object" => {
                        current_object_group
                            .objects
                            .push(current_object_attributes.clone());
                        current_object_attributes = GeneralObject::new();
                    }
                    "chunk" => {
                        current_layer.terrain_chunks.push(current_chunk.clone());
                        current_chunk = TerrainChunk::new();
                    }
                    _ => {
                        // debug!("Ending {:?}", name);
                    }
                },
                Ok(XmlEvent::Characters(text)) => {
                    if let Some(t) = &mut current_text {
                        t.text = text.clone();
                    } else {
                        for line in text.trim().lines() {
                            let tile_data: Vec<u32> = line
                                .trim()
                                .split(",")
                                .filter(|c| c.len() > 0)
                                .map(|c| c.to_string().parse().unwrap_or(0))
                                .collect();
                            current_chunk.tile_ids.push(tile_data);
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        Ok(tiled_map)
    }
}
