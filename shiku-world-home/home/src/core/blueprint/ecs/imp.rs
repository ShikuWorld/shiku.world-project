use std::collections::HashMap;

use log::error;

use crate::core::blueprint::def::ResourcePath;
use crate::core::blueprint::ecs::def::{Entity, EntityUpdate, EntityUpdateKind, ECS};
use crate::core::blueprint::scene::def::{
    GameNodeKind, GameNodeKindClean, Node2DKind, Node2DKindClean, NodeInstanceId, RenderKind,
    RenderKindClean, Scene, SceneId,
};

impl From<&Scene> for ECS {
    fn from(scene: &Scene) -> Self {
        let mut new_ecs = ECS::new();
        let mut node_id_counter = 0;
        new_ecs.scene_root = Entity(node_id_counter);
        new_ecs.scene_name = scene.name.clone();
        new_ecs.scene_resource_path = scene.resource_path.clone();
        new_ecs.scene_id = scene.id.clone();

        add_node_to_ecs(&scene.root_node, &mut new_ecs, &mut node_id_counter);

        new_ecs
    }
}

fn add_node_to_ecs(node_kind: &GameNodeKind, ecs: &mut ECS, node_id_counter: &mut NodeInstanceId) {
    let entity = Entity(*node_id_counter);
    *node_id_counter += 1;

    match node_kind {
        GameNodeKind::Node2D(node_2d) => {
            ecs.game_node_kind.insert(entity, GameNodeKindClean::Node2D);
            ecs.game_node_id.insert(entity, node_2d.id.clone());
            ecs.game_node_script.insert(entity, node_2d.script.clone());
            ecs.game_node_children.insert(entity, Vec::new());
            ecs.game_node_name.insert(entity, node_2d.name.clone());
            ecs.transforms
                .insert(entity, node_2d.data.transform.clone());

            match &node_2d.data.kind {
                Node2DKind::Node2D(_) => {
                    ecs.node_2d_kind.insert(entity, Node2DKindClean::Node2D);
                }
                Node2DKind::RigidBody(rigid_body) => {
                    ecs.node_2d_kind.insert(entity, Node2DKindClean::RigidBody);
                    ecs.rigid_body_type.insert(entity, rigid_body.body.clone());
                    ecs.rigid_body_velocity.insert(entity, rigid_body.velocity);
                }
                Node2DKind::Collider(collider) => {
                    ecs.node_2d_kind.insert(entity, Node2DKindClean::Collider);
                    ecs.collider.insert(entity, collider.clone());
                }
                Node2DKind::Render(render) => {
                    ecs.node_2d_kind.insert(entity, Node2DKindClean::Render);
                    ecs.render_layer.insert(entity, render.layer.clone());
                    ecs.render_offset.insert(entity, render.offset);
                    match render.kind {
                        RenderKind::AnimatedSprite(gid) => {
                            ecs.render_kind
                                .insert(entity, RenderKindClean::AnimatedSprite);
                            ecs.render_gid.insert(entity, gid);
                        }
                        RenderKind::Sprite(gid) => {
                            ecs.render_kind.insert(entity, RenderKindClean::Sprite);
                            ecs.render_gid.insert(entity, gid);
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
        ecs.game_node_children
            .entry(entity)
            .or_default()
            .push(Entity(*node_id_counter));
        add_node_to_ecs(child, ecs, node_id_counter);
    }
}

impl ECS {
    pub fn new() -> ECS {
        ECS {
            scene_root: Entity::default(),
            scene_name: String::default(),
            scene_resource_path: ResourcePath::default(),
            scene_id: SceneId::default(),
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
        }
    }
    pub fn apply_entity_update(&mut self, entity_update: EntityUpdate) {
        let entity = entity_update.id;
        match entity_update.kind {
            EntityUpdateKind::UpdateTransform(transform) => {
                self.transforms.insert(entity, transform);
            }
            EntityUpdateKind::UpdateGid(gid) => {
                self.render_gid.insert(entity, gid);
            }
        }
    }
}
