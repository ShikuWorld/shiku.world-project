use std::collections::HashMap;

use log::error;

use crate::core::blueprint::def::ResourcePath;
use crate::core::blueprint::ecs::def::{Entity, EntityMaps, EntityUpdate, EntityUpdateKind, ECS};
use crate::core::blueprint::scene::def::{
    GameNodeKind, GameNodeKindClean, Node2DKind, Node2DKindClean, NodeInstanceId, RenderKind,
    RenderKindClean, Scene, SceneId,
};

impl From<&Scene> for ECS {
    fn from(scene: &Scene) -> Self {
        let mut new_ecs = ECS::new();
        new_ecs.scene_root = Entity(new_ecs.entity_counter);
        new_ecs.scene_name = scene.name.clone();
        new_ecs.scene_resource_path = scene.resource_path.clone();
        new_ecs.scene_id = scene.id.clone();

        Self::add_entity(&scene.root_node, &mut new_ecs);
        new_ecs
    }
}

impl ECS {
    pub fn new() -> ECS {
        ECS {
            scene_root: Entity::default(),
            scene_name: String::default(),
            scene_resource_path: ResourcePath::default(),
            scene_id: SceneId::default(),
            entity_counter: 0,
            entities: EntityMaps {
                game_node_script_src: HashMap::new(),
                game_node_script: HashMap::new(),
                game_node_id: HashMap::new(),
                game_node_name: HashMap::new(),
                game_node_children: HashMap::new(),
                game_node_kind: HashMap::new(),
                node_2d_kind: HashMap::new(),
                render_kind: HashMap::new(),
                render_offset: HashMap::new(),
                render_layer: HashMap::new(),
                render_gid: HashMap::new(),
                transforms: HashMap::new(),
                rigid_body_velocity: HashMap::new(),
                rigid_body_type: HashMap::new(),
                rigid_body_handle: HashMap::new(),
                collider: HashMap::new(),
                collider_handle: HashMap::new(),
                dirty: HashMap::new(),
            },
        }
    }

    pub fn add_child_to_entity(
        parent_entity: Entity,
        child: &GameNodeKind,
        ecs: &mut ECS,
    ) -> Entity {
        ecs.entities
            .game_node_children
            .entry(parent_entity)
            .or_default()
            .push(Entity(ecs.entity_counter));
        Self::add_entity(child, ecs)
    }

    fn add_entity(node_kind: &GameNodeKind, ecs: &mut ECS) -> Entity {
        let entity = Entity(ecs.entity_counter);
        ecs.entity_counter += 1;

        match node_kind {
            GameNodeKind::Node2D(node_2d) => {
                ecs.entities
                    .game_node_kind
                    .insert(entity, GameNodeKindClean::Node2D);
                ecs.entities.game_node_id.insert(entity, node_2d.id.clone());
                ecs.entities
                    .game_node_script_src
                    .insert(entity, node_2d.script.clone());
                ecs.entities.game_node_children.insert(entity, Vec::new());
                ecs.entities
                    .game_node_name
                    .insert(entity, node_2d.name.clone());
                ecs.entities
                    .transforms
                    .insert(entity, node_2d.data.transform.clone());

                match &node_2d.data.kind {
                    Node2DKind::Node2D(_) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Node2D);
                    }
                    Node2DKind::RigidBody(rigid_body) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::RigidBody);
                        ecs.entities
                            .rigid_body_type
                            .insert(entity, rigid_body.body.clone());
                        ecs.entities
                            .rigid_body_velocity
                            .insert(entity, rigid_body.velocity);
                    }
                    Node2DKind::Collider(collider) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Collider);
                        ecs.entities.collider.insert(entity, collider.clone());
                    }
                    Node2DKind::Render(render) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Render);
                        ecs.entities
                            .render_layer
                            .insert(entity, render.layer.clone());
                        ecs.entities.render_offset.insert(entity, render.offset);
                        match render.kind {
                            RenderKind::AnimatedSprite(gid) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::AnimatedSprite);
                                ecs.entities.render_gid.insert(entity, gid);
                            }
                            RenderKind::Sprite(gid) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::Sprite);
                                ecs.entities.render_gid.insert(entity, gid);
                            }
                        }
                    }
                }
            }
            GameNodeKind::Instance(_node) => {
                error!("Instance not implemented!");
            }
        }
        for child in node_kind.get_children() {
            Self::add_child_to_entity(entity, child, ecs);
        }

        entity
    }

    pub fn apply_entity_update(&mut self, entity_update: EntityUpdate) {
        let entity = entity_update.id;
        match entity_update.kind {
            EntityUpdateKind::Transform(transform) => {
                self.entities.transforms.insert(entity, transform);
            }
            EntityUpdateKind::Name(name) => {
                self.entities.game_node_name.insert(entity, name);
            }
            EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                self.entities
                    .rigid_body_type
                    .insert(entity, rigid_body_type);
            }
            EntityUpdateKind::PositionRotation((x, y, r)) => {
                if let Some(transform) = self.entities.transforms.get_mut(&entity) {
                    transform.position = (x, y);
                    transform.rotation = r;
                }
            }
            EntityUpdateKind::Gid(gid) => {
                self.entities.render_gid.insert(entity, gid);
            }
        }
    }
}
