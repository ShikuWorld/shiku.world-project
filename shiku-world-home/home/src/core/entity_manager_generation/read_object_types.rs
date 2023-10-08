use crate::core::entity::physics::PhysicsType;
use crate::core::entity_manager_generation::def::{
    ObjectType, ObjectTypeProp, PropertyKind, PropertyType, RenderType,
};
use crate::resource_module::map::def::LayerName;
use log::error;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use xml::reader::XmlEvent;
use xml::EventReader;

pub fn read_object_types(object_type_path: &str) -> Vec<ObjectType> {
    let mut object_types: Vec<ObjectType> = Vec::new();

    let mut current_object_type = ObjectType {
        name: String::new(),
        props: Vec::new(),
        physics: HashMap::new(),
        physics_position: "default".to_string(),
        layer_name: LayerName::GameObjects,
        render: RenderType::NoRender,
        variants: HashMap::new(),
    };

    let mut current_object_type_prop = ObjectTypeProp {
        name: String::new(),
        property_type: PropertyType::None,
        kind: String::new(),
        default: PropertyKind::None,
    };

    let file = File::open(object_type_path).unwrap();

    let parser = EventReader::new(BufReader::new(file));

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => match name.local_name.as_str() {
                "objecttype" => {
                    for attr in attributes {
                        if attr.name.local_name.as_str() == "name" {
                            current_object_type.name = attr.value
                        }
                    }
                }
                "property" => {
                    let mut current_property_type = String::new();
                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            "name" => current_object_type_prop.name = attr.value,
                            "type" => current_object_type_prop.kind = attr.value,
                            "default" => match current_object_type_prop.kind.as_str() {
                                "float" => {
                                    current_object_type_prop.default =
                                        PropertyKind::Float(attr.value.parse().unwrap());
                                }
                                "int" => {
                                    current_object_type_prop.default =
                                        PropertyKind::Int(attr.value.parse().unwrap());
                                }
                                "string" => {
                                    current_object_type_prop.default =
                                        PropertyKind::String(attr.value);
                                }
                                "bool" => {
                                    current_object_type_prop.default =
                                        PropertyKind::Boolean(attr.value != "false");
                                }
                                "object" => {
                                    current_object_type_prop.default =
                                        PropertyKind::Object(attr.value);
                                }
                                "class" => {
                                    current_object_type_prop.default =
                                        PropertyKind::Class(attr.value);
                                }
                                _ => {
                                    error!(
                                        "Undefined type '{:?}' for property default",
                                        attr.value
                                    );
                                }
                            },
                            "propertytype" => {
                                current_property_type = attr.value;
                            }
                            _ => (),
                        }

                        if current_object_type_prop.name == "physics_position" {
                            if let PropertyKind::String(value) = &current_object_type_prop.default {
                                current_object_type.physics_position = value.clone();
                            }
                        }

                        match current_property_type.as_str() {
                            "LayerName" => {
                                if let PropertyKind::String(layer_name) =
                                    &current_object_type_prop.default
                                {
                                    match layer_name.as_str() {
                                        "Guest" => {
                                            current_object_type.layer_name = LayerName::Guest
                                        }
                                        "FG0" => current_object_type.layer_name = LayerName::FG0,
                                        "FG1" => current_object_type.layer_name = LayerName::FG1,
                                        "FG2" => current_object_type.layer_name = LayerName::FG2,
                                        "FG3" => current_object_type.layer_name = LayerName::FG3,
                                        "FG4" => current_object_type.layer_name = LayerName::FG4,
                                        "FG5" => current_object_type.layer_name = LayerName::FG5,
                                        "FG6" => current_object_type.layer_name = LayerName::FG6,
                                        "FG7" => current_object_type.layer_name = LayerName::FG7,
                                        "FG8" => current_object_type.layer_name = LayerName::FG8,
                                        "FG9" => current_object_type.layer_name = LayerName::FG9,
                                        "FG10" => current_object_type.layer_name = LayerName::FG10,
                                        "FG11" => current_object_type.layer_name = LayerName::FG11,
                                        "Menu" => current_object_type.layer_name = LayerName::Menu,
                                        "GameObjects" => {
                                            current_object_type.layer_name = LayerName::GameObjects
                                        }
                                        "BG0" => current_object_type.layer_name = LayerName::BG0,
                                        "BG1" => current_object_type.layer_name = LayerName::BG1,
                                        "BG2" => current_object_type.layer_name = LayerName::BG2,
                                        "BG3" => current_object_type.layer_name = LayerName::BG3,
                                        "BG4" => current_object_type.layer_name = LayerName::BG4,
                                        "BG5" => current_object_type.layer_name = LayerName::BG5,
                                        "BG6" => current_object_type.layer_name = LayerName::BG6,
                                        "BG7" => current_object_type.layer_name = LayerName::BG7,
                                        "BG8" => current_object_type.layer_name = LayerName::BG8,
                                        "BG9" => current_object_type.layer_name = LayerName::BG9,
                                        "BG10" => current_object_type.layer_name = LayerName::BG10,
                                        "BG11" => current_object_type.layer_name = LayerName::BG11,
                                        "Terrain" => {
                                            current_object_type.layer_name = LayerName::Terrain
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            "Physics" => {
                                current_object_type_prop.property_type =
                                    match &current_object_type_prop.default {
                                        PropertyKind::String(value) => match value.as_str() {
                                            "RigidBody" => {
                                                PropertyType::Physics(PhysicsType::RigidBody)
                                            }
                                            "StaticRigidBody" => {
                                                PropertyType::Physics(PhysicsType::StaticRigidBody)
                                            }
                                            "Area" => PropertyType::Physics(PhysicsType::Area),
                                            _ => PropertyType::Physics(PhysicsType::None),
                                        },
                                        _ => PropertyType::Physics(PhysicsType::None),
                                    }
                            }
                            "Tween" => {
                                current_object_type_prop.default =
                                    PropertyKind::Class("Tween".to_string());

                                current_object_type_prop.property_type = PropertyType::Tween;
                            }
                            "Render" => {
                                current_object_type_prop.property_type =
                                    match &current_object_type_prop.default {
                                        PropertyKind::String(value) => match value.as_str() {
                                            "StaticImage" => {
                                                PropertyType::Render(RenderType::StaticImage)
                                            }
                                            "Text" => {
                                                PropertyType::Render(RenderType::RenderTypeText)
                                            }
                                            "Timer" => {
                                                PropertyType::Render(RenderType::RenderTypeTimer)
                                            }
                                            "None" => PropertyType::Render(RenderType::NoRender),
                                            _ => PropertyType::Render(RenderType::NoRender),
                                        },
                                        _ => PropertyType::Render(RenderType::NoRender),
                                    }
                            }
                            _ => (),
                        };

                        if let PropertyType::Physics(physics_type) =
                            &current_object_type_prop.property_type
                        {
                            if current_object_type_prop.name.contains("physics_col_") {
                                current_object_type.physics.insert(
                                    current_object_type_prop.name[12..].to_string(),
                                    physics_type.clone(),
                                );
                            }

                            if current_object_type_prop.name == "physics" {
                                current_object_type
                                    .physics
                                    .insert("default".to_string(), physics_type.clone());
                            }
                        }

                        if let PropertyType::Render(render_type) =
                            &current_object_type_prop.property_type
                        {
                            current_object_type.render = render_type.clone();
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::EndElement { name }) => match name.local_name.as_str() {
                "objecttype" => {
                    if current_object_type.name != "Tween" {
                        object_types.push(current_object_type);
                    }
                    current_object_type = ObjectType {
                        name: String::new(),
                        props: Vec::new(),
                        physics: HashMap::new(),
                        physics_position: "default".to_string(),
                        layer_name: LayerName::GameObjects,
                        render: RenderType::NoRender,
                        variants: HashMap::new(),
                    };
                }
                "property" => {
                    current_object_type.props.push(current_object_type_prop);
                    current_object_type_prop = ObjectTypeProp {
                        name: String::new(),
                        property_type: PropertyType::None,
                        kind: String::new(),
                        default: PropertyKind::None,
                    };
                }
                _ => {
                    // debug!("Ending {:?}", name);
                }
            },
            Err(e) => {
                error!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    object_types
}
