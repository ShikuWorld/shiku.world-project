use crate::core::blueprint::ecs::def::{Entity, ECS};
use crate::core::blueprint::scene::def::{
    GameNode, GameNodeKind, GameNodeKindClean, Node2D, Node2DDud, Node2DKind, Node2DKindClean,
    Render, RenderKind, RenderKindClean, RigidBody, Scene,
};
use log::error;

pub fn build_scene_from_ecs(ecs: &ECS) -> Option<Scene> {
    let root_entity = &ecs.scene_root;
    if let Some(root_node) = get_game_node_kind_from_ecs(root_entity, ecs) {
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

fn get_game_node_kind_from_ecs(entity: &Entity, ecs: &ECS) -> Option<GameNodeKind> {
    if let (
        Some(node_kind),
        Some(node_id),
        Some(node_name),
        Some(node_script),
        Some(node_children),
    ) = (
        ecs.game_node_kind.get(entity),
        ecs.game_node_id.get(entity),
        ecs.game_node_name.get(entity),
        ecs.game_node_script.get(entity),
        ecs.game_node_children.get(entity),
    ) {
        let children: Vec<GameNodeKind> = node_children
            .into_iter()
            .filter_map(|child_entity| get_game_node_kind_from_ecs(child_entity, ecs))
            .collect();
        match node_kind {
            GameNodeKindClean::Instance => {
                return Some(GameNodeKind::Instance(GameNode {
                    id: node_id.clone(),
                    name: node_name.clone(),
                    script: Some(node_script.clone()),
                    entity_id: Some(*entity),
                    children,
                    data: "".into(),
                }));
            }
            GameNodeKindClean::Node2D => {
                if let (Some(node_2d_kind), Some(transform)) = (
                    get_render_node_2d_kind_from_ecs(entity, ecs),
                    ecs.transforms.get(entity),
                ) {
                    return Some(GameNodeKind::Node2D(GameNode {
                        id: node_id.clone(),
                        name: node_name.clone(),
                        script: Some(node_script.clone()),
                        entity_id: Some(*entity),
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
    error!("Was not able to get game_node...?");
    None
}

fn get_render_node_2d_kind_from_ecs(entity: &Entity, ecs: &ECS) -> Option<Node2DKind> {
    if let Some(node_2d_kind) = ecs.node_2d_kind.get(entity) {
        match node_2d_kind {
            Node2DKindClean::Node2D => return Some(Node2DKind::Node2D(Node2DDud(0))),
            Node2DKindClean::RigidBody => {
                if let (Some(velocity), Some(body)) = (
                    ecs.rigid_body_velocity.get(entity),
                    ecs.rigid_body_type.get(entity),
                ) {
                    return Some(Node2DKind::RigidBody(RigidBody {
                        body: body.clone(),
                        velocity: velocity.clone(),
                    }));
                }
            }
            Node2DKindClean::Collider => {
                if let Some(collider) = ecs.collider.get(entity) {
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
    None
}

fn get_render_from_ecs(entity: &Entity, ecs: &ECS) -> Option<Render> {
    if let (Some(render_kind), Some(render_layer), Some(render_offset)) = (
        ecs.render_kind.get(entity),
        ecs.render_layer.get(entity),
        ecs.render_offset.get(entity),
    ) {
        if let Some(kind) = match render_kind {
            RenderKindClean::AnimatedSprite => ecs
                .render_gid
                .get(entity)
                .map(|gid| RenderKind::AnimatedSprite(*gid)),
            RenderKindClean::Sprite => ecs
                .render_gid
                .get(entity)
                .map(|gid| RenderKind::Sprite(*gid)),
        } {
            return Some(Render {
                offset: render_offset.clone(),
                layer: render_layer.clone(),
                kind,
            });
        }
    }
    None
}
impl GameNodeKind {
    pub fn borrow_children(&mut self) -> &mut Vec<GameNodeKind> {
        match self {
            GameNodeKind::Instance(node) => &mut node.children,
            GameNodeKind::Node2D(node) => &mut node.children,
        }
    }

    pub fn get_children(&self) -> &Vec<GameNodeKind> {
        match self {
            GameNodeKind::Instance(node) => &node.children,
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
                "Tried to remove a child that was not there, this could have paniced! len: {:?} | index: {:?}",
                children.len(),
                index
            );
        }
    }

    pub fn set_data(&mut self, data: GameNodeKind) {
        match self {
            GameNodeKind::Instance(node) => {
                if let GameNodeKind::Instance(n) = data {
                    node.data = n.data
                }
            }
            GameNodeKind::Node2D(node) => {
                if let GameNodeKind::Node2D(n) = data {
                    node.data = n.data
                }
            }
        }
    }
}
