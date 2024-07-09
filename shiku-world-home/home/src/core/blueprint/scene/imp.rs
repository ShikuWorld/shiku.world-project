use crate::core::blueprint::def::JsonResource;
use log::{debug, error};
use std::collections::HashMap;

use crate::core::blueprint::ecs::def::{ECSShared, Entity, EntityUpdateKind, ECS};
use crate::core::blueprint::ecs::game_node_script::GameNodeScript;
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{
    GameNode, GameNodeKind, GameNodeKindClean, KinematicCharacterControllerProps, Node2D,
    Node2DDud, Node2DKind, Node2DKindClean, Render, RenderKind, RenderKindClean, RigidBody,
    RigidBodyType, Scene,
};

pub fn build_scene_from_ecs(ecs: &ECS) -> Option<Scene> {
    let root_entity = &ecs.scene_root;
    if let Some(root_node) = GameNodeKind::get_game_node_kind_from_ecs(root_entity, ecs) {
        let scene = Scene {
            id: ecs.scene_name.clone(),
            resource_path: ecs.scene_resource_path.clone(),
            name: ecs.scene_name.clone(),
            root_node,
        };

        return Some(scene);
    }
    error!("Could not load scene from ecs...?");
    None
}

fn get_render_node_2d_kind_from_ecs(entity: &Entity, ecs: &ECSShared) -> Option<Node2DKind> {
    if let Some(node_2d_kind) = ecs.entities.node_2d_kind.get(entity) {
        match node_2d_kind {
            Node2DKindClean::Instance => {
                if let Some(instance_path) = ecs.entities.node_2d_instance_path.get(entity) {
                    return Some(Node2DKind::Instance(instance_path.clone()));
                }
            }
            Node2DKindClean::Node2D => return Some(Node2DKind::Node2D(Node2DDud(0))),
            Node2DKindClean::RigidBody => {
                if let Some(body) = ecs.entities.rigid_body_type.get(entity) {
                    return Some(Node2DKind::RigidBody(RigidBody {
                        body: body.clone(),
                        kinematic_character_controller_props: ecs
                            .entities
                            .kinematic_character
                            .get(entity)
                            .map(|k| k.props.clone()),
                    }));
                }
            }
            Node2DKindClean::Collider => {
                if let Some(collider) = ecs.entities.collider.get(entity) {
                    return Some(Node2DKind::Collider(collider.clone()));
                }
            }
            Node2DKindClean::Render => {
                if let Some(render) = get_render_from_ecs(entity, ecs) {
                    return Some(Node2DKind::Render(render));
                }
            }
        }
    }
    error!(
        "Was not able to get node_2d_kind. entity: {:?}, node_2d_kind: {:?}",
        entity,
        ecs.entities.node_2d_kind.get(entity)
    );
    None
}

fn get_render_from_ecs(entity: &Entity, ecs: &ECSShared) -> Option<Render> {
    if let (Some(render_kind), Some(render_layer), Some(render_offset)) = (
        ecs.entities.render_kind.get(entity),
        ecs.entities.render_layer.get(entity),
        ecs.entities.render_offset.get(entity),
    ) {
        if let Some(kind) = match render_kind {
            RenderKindClean::AnimatedSprite => {
                ecs.entities
                    .character_animation
                    .get(entity)
                    .map(|character_animation| {
                        RenderKind::AnimatedSprite(
                            character_animation.blueprint.get_full_resource_path(),
                            character_animation.current_gid,
                        )
                    })
            }
            RenderKindClean::Sprite => ecs.entities.render_gid.get(entity).and_then(|gid| {
                ecs.entities
                    .render_gid_tileset_path
                    .get(entity)
                    .map(|tileset_path| RenderKind::Sprite(tileset_path.clone(), *gid))
            }),
        } {
            return Some(Render {
                offset: *render_offset,
                layer: render_layer.clone(),
                kind,
            });
        }
    }
    error!("Was not able to get render_node. entity: {:?}, render_kind: {:?}, layer: {:?}, offset: {:?}",
        entity,
        ecs.entities.render_kind.get(entity),
        ecs.entities.render_layer.get(entity),
        ecs.entities.render_offset.get(entity));
    None
}
impl GameNodeKind {
    pub fn borrow_children(&mut self) -> &mut Vec<GameNodeKind> {
        match self {
            GameNodeKind::Node2D(node) => &mut node.children,
        }
    }

    pub fn get_children(&self) -> &Vec<GameNodeKind> {
        match self {
            GameNodeKind::Node2D(node) => &node.children,
        }
    }

    pub fn add_child(&mut self, other_game_node: GameNodeKind) {
        self.borrow_children().push(other_game_node)
    }

    pub fn remove_child(&mut self, index: usize) {
        let children = self.borrow_children();
        if index < children.len() {
            children.remove(index);
        } else {
            error!(
                "Tried to remove a child that was not there, this could have panicked! len: {:?} | index: {:?}",
                children.len(),
                index
            );
        }
    }

    pub fn update_with_entity_update(&mut self, update: EntityUpdateKind) {
        let GameNodeKind::Node2D(n) = self;
        match update {
            EntityUpdateKind::Tags(tags) => {
                n.tags = tags;
            }
            EntityUpdateKind::InstancePath(instance_path) => {
                if let Node2DKind::Instance(path) = &mut n.data.kind {
                    *path = instance_path;
                }
            }
            EntityUpdateKind::KinematicCharacterControllerProps(props) => {
                if let Node2DKind::RigidBody(rigid_body) = &mut n.data.kind {
                    rigid_body.kinematic_character_controller_props = Some(props);
                }
            }
            EntityUpdateKind::Transform(transform) => {
                n.data.transform = transform;
            }
            EntityUpdateKind::ScriptPath(script_path_option) => {
                n.script = script_path_option;
            }
            EntityUpdateKind::Name(name) => {
                n.name = name;
            }
            EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                if let Node2DKind::RigidBody(rigid_body) = &mut n.data.kind {
                    rigid_body.body = rigid_body_type;
                    rigid_body.kinematic_character_controller_props = match rigid_body.body {
                        RigidBodyType::KinematicPositionBased
                        | RigidBodyType::KinematicVelocityBased => {
                            Some(KinematicCharacterControllerProps::new())
                        }
                        RigidBodyType::Dynamic | RigidBodyType::Fixed => None,
                    }
                }
            }
            EntityUpdateKind::RenderKind(render_kind) => {
                if let Node2DKind::Render(render) = &mut n.data.kind {
                    render.kind = render_kind;
                }
            }
            EntityUpdateKind::PositionRotation((x, y, r)) => {
                n.data.transform.position = (x, y);
                n.data.transform.rotation = r;
            }
            EntityUpdateKind::Collider(collider) => {
                if let Node2DKind::Collider(c) = &mut n.data.kind {
                    *c = collider;
                }
            }
            EntityUpdateKind::AnimatedSpriteResource(resource_path) => {
                if let Node2DKind::Render(r) = &mut n.data.kind {
                    if let RenderKind::AnimatedSprite(ref mut r, _) = r.kind {
                        *r = resource_path.clone();
                    }
                }
            }
            EntityUpdateKind::Gid(gid) => {
                if let Node2DKind::Render(r) = &mut n.data.kind {
                    match r.kind {
                        RenderKind::AnimatedSprite(_, ref mut g) => {
                            *g = gid;
                        }
                        RenderKind::Sprite(_, ref mut g) => {
                            *g = gid;
                        }
                    }
                }
            }
            EntityUpdateKind::SpriteTilesetResource(resource_path) => {
                if let Node2DKind::Render(render) = &mut n.data.kind {
                    match render.kind {
                        RenderKind::AnimatedSprite(ref mut r, _) => {
                            *r = resource_path;
                        }
                        RenderKind::Sprite(ref mut r, _) => {
                            *r = resource_path;
                        }
                    }
                }
            }
            EntityUpdateKind::UpdateScriptScope(scope_key, scope_value) => {
                debug!(
                    "Update script scope not implemented for scenes: {:?} {:?}",
                    scope_key, scope_value
                );
            }
            EntityUpdateKind::SetScriptScope(scope_cache) => {
                debug!(
                    "Set script scope not implemented for scenes: {:?}",
                    scope_cache
                );
            }
        }
    }

    pub(crate) fn get_game_node_kind_from_ecs(
        original_entity: &Entity,
        ecs: &ECS,
    ) -> Option<GameNodeKind> {
        if let Some(shared) = ecs.shared.try_borrow_mut() {
            return GameNodeKind::_get_game_node_kind_from_ecs(
                original_entity,
                &ecs.entity_scripts,
                &shared,
            );
        }

        None
    }

    fn _get_game_node_kind_from_ecs(
        original_entity: &Entity,
        entity_scripts: &HashMap<Entity, GameNodeScript>,
        shared: &ECSShared,
    ) -> Option<GameNodeKind> {
        let mut possible_instance_root = original_entity;
        if let Some(root_entity) = shared.get_instance_root_entity(possible_instance_root) {
            possible_instance_root = root_entity;
        }
        if let (Some(node_kind), Some(node_id), Some(node_name), Some(node_children)) = (
            shared.entities.game_node_kind.get(possible_instance_root),
            shared.entities.game_node_id.get(possible_instance_root),
            shared.entities.game_node_name.get(original_entity),
            shared
                .entities
                .game_node_children
                .get(possible_instance_root),
        ) {
            let children: Vec<GameNodeKind> = node_children
                .iter()
                .filter_map(|child_entity| {
                    GameNodeKind::_get_game_node_kind_from_ecs(child_entity, entity_scripts, shared)
                })
                .collect();
            match node_kind {
                GameNodeKindClean::Node2D => {
                    if let (Some(node_2d_kind), Some(transform)) = (
                        get_render_node_2d_kind_from_ecs(possible_instance_root, shared),
                        shared.entities.transforms.get(possible_instance_root),
                    ) {
                        return Some(GameNodeKind::Node2D(GameNode {
                            id: node_id.clone(),
                            name: node_name.clone(),
                            script: entity_scripts
                                .get(possible_instance_root)
                                .map(|s| s.path.clone()),
                            entity_id: Some(*possible_instance_root),
                            tags: shared
                                .entities
                                .game_node_tags
                                .get(original_entity)
                                .cloned()
                                .unwrap_or_default(),
                            instance_resource_path: shared
                                .entities
                                .node_2d_instance_path
                                .get(original_entity)
                                .cloned(),
                            children,
                            data: Node2D {
                                transform: transform.clone(),
                                kind: node_2d_kind,
                            },
                        }));
                    }
                }
            }
        }

        error!("Was not able to get game_node. entity:, kind: {:?}, id: {:?}, name: {:?}, script: {:?}, children: {:?}",
        original_entity,
        shared.entities.game_node_kind.get(original_entity),
        shared.entities.game_node_id.get(original_entity),
        shared.entities.game_node_name.get(original_entity),
        shared.entities.game_node_children.contains_key(original_entity));
        None
    }
}
