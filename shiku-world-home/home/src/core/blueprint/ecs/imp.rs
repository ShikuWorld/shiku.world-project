use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};

use log::{debug, error};
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::ColliderHandle;
use rapier2d::math::Vector;
use rhai::{Engine, Scope, AST};

use crate::core::blueprint::def::ResourcePath;
use crate::core::blueprint::ecs::character_animation::CharacterAnimation;
use crate::core::blueprint::ecs::def::{
    DynamicRigidBodyPropsUpdate, ECSShared, Entity, EntityMaps, EntityUpdate, EntityUpdateKind,
    KinematicCharacter, TimerId, TweenId, ECS,
};
use crate::core::blueprint::ecs::game_node_script::{
    GameNodeScript, GameNodeScriptFunction, ScopeCacheValue,
};
use crate::core::blueprint::resource_loader::Blueprint;
use crate::core::blueprint::scene::def::{
    Collider, ColliderKind, ColliderShape, DynamicRigidBodyProps, GameNodeKind, GameNodeKindClean,
    KinematicCharacterControllerProps, Node2DKind, Node2DKindClean, ProgressBar, RenderKind,
    RenderKindClean, RigidBodyType, Scene, SceneId, Transform,
};
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::timer::Timer;
use crate::core::tween::Tween;
use crate::core::ApiShare;

impl From<&Scene> for ECS {
    fn from(scene: &Scene) -> Self {
        let mut new_ecs = ECS::new();
        if let Some(mut shared) = new_ecs.shared.try_borrow_mut() {
            new_ecs.scene_root = Entity(shared.entity_counter);
            new_ecs.scene_name.clone_from(&scene.name);
            new_ecs.scene_resource_path.clone_from(&scene.resource_path);
            new_ecs.scene_id.clone_from(&scene.id);

            Self::add_entity_from_game_node(&scene.root_node, &mut shared);
        }
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
            entity_scripts: HashMap::new(),
            entities: HashSet::new(),
            processed_added_entities: Vec::new(),
            intersects_data_tmp: Vec::new(),
            script_pending_removal: HashSet::new(),
            shared: ApiShare::new(ECSShared {
                entities: EntityMaps {
                    game_node_id: HashMap::new(),
                    game_node_name: HashMap::new(),
                    game_node_children: HashMap::new(),
                    game_node_kind: HashMap::new(),
                    game_node_tags: HashMap::new(),
                    node_2d_kind: HashMap::new(),
                    node_2d_instance_path: HashMap::new(),
                    node_2d_entity_instance_parent: HashMap::new(),
                    game_node_parent: HashMap::new(),
                    render_kind: HashMap::new(),
                    render_offset: HashMap::new(),
                    render_layer: HashMap::new(),
                    render_gid: HashMap::new(),
                    ui_progress_bar: HashMap::new(),
                    render_fadeout_effect: HashMap::new(),
                    render_fadein_effect: HashMap::new(),
                    render_gid_tileset_path: HashMap::new(),
                    character_animation: HashMap::new(),
                    transforms: HashMap::new(),
                    kinematic_character: HashMap::new(),
                    rigid_body_type: HashMap::new(),
                    rigid_body_handle: HashMap::new(),
                    dynamic_rigid_body_props: HashMap::new(),
                    collider: HashMap::new(),
                    collider_handle: HashMap::new(),
                    text_render: HashMap::new(),
                    dirty: HashMap::new(),
                    view_dirty: HashMap::new(),
                },
                tween_map: HashMap::new(),
                timer_map: HashMap::new(),
                timer_counter: 0,
                tween_counter: 0,
                set_scope_variables: HashMap::new(),
                added_entities: Vec::new(),
                removed_entities: Vec::new(),
                character_collisions_tmp: Vec::new(),
                kinematic_collision_map: HashMap::new(),
                collider_to_entity_map: HashMap::new(),
                collider_to_parent_entity_map: HashMap::new(),
                removed_colliders: Vec::new(),
                entity_counter: 0,
            }),
        }
    }

    pub fn create_initial_rigid_bodies(&mut self, physics: &mut RapierSimulation) {
        if let Some(mut shared) = self.shared.try_borrow_mut() {
            for (original_entity, rigid_body_type) in shared.entities.rigid_body_type.clone() {
                let possible_instance_entity =
                    { Self::get_instance_entity_if_exists(&original_entity, &shared) };
                let transform = shared
                    .entities
                    .transforms
                    .get(&possible_instance_entity)
                    .cloned()
                    .unwrap_or_default();
                Self::add_rigid_body_for_entity(
                    &original_entity,
                    &rigid_body_type,
                    &transform,
                    &mut shared,
                    physics,
                );
            }
        }
    }

    pub fn add_rigid_body_for_entity(
        original_entity: &Entity,
        rigid_body_type: &RigidBodyType,
        transform: &Transform,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        let rigid_body_handle =
            Self::create_rigid_body_from_type(rigid_body_type, transform, physics);
        if let Some(dynamic_rigid_body_props) = shared
            .entities
            .dynamic_rigid_body_props
            .get(original_entity)
        {
            physics.set_dynamic_rigid_body_props(
                rigid_body_handle,
                DynamicRigidBodyPropsUpdate {
                    can_sleep: Some(dynamic_rigid_body_props.can_sleep),
                    rotation_locked: Some(dynamic_rigid_body_props.rotation_locked),
                    angular_dampening: Some(dynamic_rigid_body_props.angular_dampening),
                    linear_dampening: Some(dynamic_rigid_body_props.linear_dampening),
                    gravity_scale: Some(dynamic_rigid_body_props.gravity_scale),
                    ccd_enabled: Some(dynamic_rigid_body_props.ccd_enabled),
                },
            );
        }
        shared
            .entities
            .rigid_body_handle
            .insert(*original_entity, rigid_body_handle);
    }

    fn get_instance_entity_if_exists(original_entity: &Entity, shared: &ECSShared) -> Entity {
        if let Some(parent_entity) = shared
            .entities
            .node_2d_entity_instance_parent
            .get(original_entity)
        {
            return *parent_entity;
        }

        *original_entity
    }

    fn create_rigid_body_from_type(
        rigid_body_type: &RigidBodyType,
        transform: &Transform,
        physics: &mut RapierSimulation,
    ) -> RigidBodyHandle {
        match rigid_body_type {
            RigidBodyType::Dynamic => {
                physics.add_dynamic_rigid_body(transform.position.0, transform.position.1)
            }
            RigidBodyType::Fixed => {
                physics.add_fixed_rigid_body(transform.position.0, transform.position.1)
            }
            RigidBodyType::KinematicPositionBased => physics
                .add_kinematic_position_based_rigid_body(
                    transform.position.0,
                    transform.position.1,
                ),
            RigidBodyType::KinematicVelocityBased => physics
                .add_kinematic_velocity_based_rigid_body(
                    transform.position.0,
                    transform.position.1,
                ),
        }
    }

    pub fn attach_colliders_to_entity(
        entity: &Entity,
        ecs: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        if let (Some(children), Some(rigid_body_handle)) = (
            ecs.entities.game_node_children.get(entity),
            ecs.entities.rigid_body_handle.get(entity),
        ) {
            for child_entity in children {
                if let Some(child_collider) = ecs.entities.collider.get(child_entity) {
                    let child_collider_handle =
                        Self::create_collider(child_collider, rigid_body_handle, physics);
                    ecs.entities
                        .collider_handle
                        .insert(*child_entity, child_collider_handle);
                    ecs.collider_to_entity_map
                        .insert(child_collider_handle, *child_entity);
                    ecs.collider_to_parent_entity_map
                        .insert(child_collider_handle, *entity);
                }
            }
        }
    }

    pub fn attach_collider_to_its_entity(
        parent_entity: &Entity,
        child_entity: &Entity,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        if let (Some(child_collider), Some(parent_rigid_body_handle)) = (
            shared.entities.collider.get(child_entity),
            shared.entities.rigid_body_handle.get(parent_entity),
        ) {
            let child_collider_handle =
                Self::create_collider(child_collider, parent_rigid_body_handle, physics);
            shared
                .entities
                .collider_handle
                .insert(*child_entity, child_collider_handle);
            shared
                .collider_to_entity_map
                .insert(child_collider_handle, *child_entity);
            shared
                .collider_to_parent_entity_map
                .insert(child_collider_handle, *parent_entity);
        }
    }

    pub fn attach_initial_colliders_to_rigid_bodies(&mut self, physics: &mut RapierSimulation) {
        if let Some(mut shared) = self.shared.try_borrow_mut() {
            Self::_attach_initial_colliders_to_rigid_bodies(&mut shared, physics);
        }
    }

    fn _attach_initial_colliders_to_rigid_bodies(
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
    ) {
        for (parent_entity, children) in &shared.entities.game_node_children {
            if let Some(rigid_body_handle) = shared.entities.rigid_body_handle.get(parent_entity) {
                for child_entity in children {
                    if let Some(child_collider) = shared.entities.collider.get(child_entity) {
                        let child_collider_handle =
                            Self::create_collider(child_collider, rigid_body_handle, physics);
                        shared
                            .entities
                            .collider_handle
                            .insert(*child_entity, child_collider_handle);
                        shared
                            .collider_to_entity_map
                            .insert(child_collider_handle, *child_entity);
                        shared
                            .collider_to_parent_entity_map
                            .insert(child_collider_handle, *parent_entity);
                    }
                }
            }
        }
    }

    fn create_collider(
        collider: &Collider,
        rigid_body_handle: &RigidBodyHandle,
        physics: &mut RapierSimulation,
    ) -> ColliderHandle {
        let is_sensor = match collider.kind {
            ColliderKind::Solid => false,
            ColliderKind::Sensor => true,
        };
        match collider.shape {
            ColliderShape::Ball(radius) => physics.create_ball_collider(
                radius,
                *rigid_body_handle,
                is_sensor,
                collider.density,
                collider.restitution,
            ),
            ColliderShape::CapsuleX(half_y, radius) => {
                physics.create_capsule_x_collider(half_y, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::CapsuleY(half_x, radius) => {
                physics.create_capsule_y_collider(half_x, radius, *rigid_body_handle, is_sensor)
            }
            ColliderShape::Cuboid(half_x, half_y) => physics.create_cuboid_collider(
                half_x,
                half_y,
                *rigid_body_handle,
                is_sensor,
                collider.density,
                collider.restitution,
            ),
        }
    }

    pub fn add_child_to_entity(
        parent_entity: Entity,
        child: &GameNodeKind,
        shared: &mut ECSShared,
    ) -> Entity {
        let child_entity = Entity(shared.entity_counter);
        shared
            .entities
            .game_node_children
            .entry(parent_entity)
            .or_default()
            .push(child_entity);
        shared
            .entities
            .game_node_parent
            .insert(child_entity, parent_entity);
        if let Some(Node2DKindClean::Instance) = shared.entities.node_2d_kind.get(&parent_entity) {
            shared
                .entities
                .node_2d_entity_instance_parent
                .insert(child_entity, parent_entity);
        }
        Self::add_entity_from_game_node(child, shared)
    }

    fn add_entity_from_game_node(node_kind: &GameNodeKind, ecs: &mut ECSShared) -> Entity {
        let entity = Entity(ecs.entity_counter);
        let mut script_path = None;
        ecs.entity_counter += 1;
        match node_kind {
            GameNodeKind::Node2D(node_2d) => {
                ecs.entities
                    .game_node_kind
                    .insert(entity, GameNodeKindClean::Node2D);
                ecs.entities.game_node_id.insert(entity, node_2d.id.clone());
                script_path.clone_from(&node_2d.script);
                ecs.entities.game_node_children.insert(entity, Vec::new());
                ecs.entities
                    .game_node_tags
                    .insert(entity, node_2d.tags.clone());
                ecs.entities
                    .game_node_name
                    .insert(entity, node_2d.name.clone());
                ecs.entities
                    .transforms
                    .insert(entity, node_2d.data.transform.clone());

                match &node_2d.data.kind {
                    Node2DKind::Instance(resource_path) => {
                        ecs.entities
                            .node_2d_kind
                            .insert(entity, Node2DKindClean::Instance);
                        ecs.entities
                            .node_2d_instance_path
                            .insert(entity, resource_path.clone());
                    }
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
                        if let Some(kinematic_character_controller_props) =
                            &rigid_body.kinematic_character_controller_props
                        {
                            ecs.entities.kinematic_character.insert(
                                entity,
                                KinematicCharacter {
                                    controller:
                                        RapierSimulation::create_kinematic_character_controller(
                                            kinematic_character_controller_props,
                                        ),
                                    props: kinematic_character_controller_props.clone(),
                                    desired_translation: Vector::zeros(),
                                    is_sliding_down_slope: false,
                                    grounded: false,
                                },
                            );
                        }
                        if let Some(dynamic_rigid_body_props) = &rigid_body.dynamic_rigid_body_props
                        {
                            ecs.entities
                                .dynamic_rigid_body_props
                                .insert(entity, dynamic_rigid_body_props.clone());
                        }
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
                        ecs.entities
                            .render_fadein_effect
                            .insert(entity, render.fadein_effect.clone());
                        ecs.entities
                            .render_fadeout_effect
                            .insert(entity, render.fadeout_effect.clone());
                        match &render.kind {
                            RenderKind::Text(text_render) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::Text);
                                ecs.entities.text_render.insert(entity, text_render.clone());
                            }
                            RenderKind::AnimatedSprite(resource_path, _) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::AnimatedSprite);
                                Self::add_character_animation(ecs, entity, resource_path);
                            }
                            RenderKind::Sprite(resource_path, gid) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::Sprite);
                                ecs.entities.render_gid.insert(entity, *gid);
                                ecs.entities
                                    .render_gid_tileset_path
                                    .insert(entity, resource_path.clone());
                            }
                            RenderKind::ProgressBar(progress_bar) => {
                                ecs.entities
                                    .render_kind
                                    .insert(entity, RenderKindClean::ProgressBar);
                                ecs.entities
                                    .ui_progress_bar
                                    .insert(entity, progress_bar.clone());
                            }
                        }
                    }
                }
            }
        }
        ecs.added_entities.push((entity, script_path));
        if let Some(instance_root_node) = Self::get_node_2d_instance_root_node(node_kind) {
            debug!("# Adding instance root node");
            Self::add_child_to_entity(entity, &instance_root_node, ecs);
        } else {
            for child in node_kind.get_children() {
                Self::add_child_to_entity(entity, child, ecs);
            }
        }

        entity
    }

    fn add_character_animation(ecs: &mut ECSShared, entity: Entity, resource_path: &ResourcePath) {
        match Blueprint::load_character_animation(resource_path.into()) {
            Ok(character_animation) => {
                let c_a: CharacterAnimation = character_animation.into();
                ecs.entities.render_gid.insert(entity, c_a.current_gid);
                ecs.entities
                    .render_kind
                    .insert(entity, RenderKindClean::AnimatedSprite);
                ecs.entities.character_animation.insert(entity, c_a);
            }
            Err(e) => {
                error!("Error loading character animation: {:?}", e);
            }
        }
    }

    fn get_node_2d_instance_root_node(game_node_kind: &GameNodeKind) -> Option<GameNodeKind> {
        match game_node_kind {
            GameNodeKind::Node2D(node_2d) => match &node_2d.data.kind {
                Node2DKind::Instance(resource_path) => {
                    match Blueprint::load_scene(resource_path.clone().into()) {
                        Ok(scene) => Some(scene.root_node),
                        Err(e) => {
                            error!("Error loading scene: {:?}", e);
                            None
                        }
                    }
                }
                _ => None,
            },
        }
    }

    pub fn remove_script_on_all_entities(&mut self, resource_path: &ResourcePath) {
        self.entity_scripts
            .retain(|_, script| script.path != *resource_path);
    }

    pub fn update_tweens_and_timers(shared: &mut ECSShared, time_update: f64) {
        for tween in shared.tween_map.values_mut() {
            tween.update(time_update);
        }
        for timer in shared.timer_map.values_mut() {
            timer.update(time_update);
        }
    }

    pub fn process_added_and_removed_entities_and_scope_sets(&mut self, engine: &Engine) {
        let mut processing_done = false;
        while !processing_done {
            if let Some(mut shared) = self.shared.try_borrow_mut() {
                for (new_entity, resource_path) in shared
                    .added_entities
                    .drain(..)
                    .filter_map(|(e, p)| p.map(|r| (e, r)))
                {
                    match GameNodeScript::new(new_entity, &Engine::new(), resource_path.clone()) {
                        Ok(game_node_script) => {
                            self.entity_scripts.insert(new_entity, game_node_script);
                            self.processed_added_entities.push(new_entity);
                        }
                        Err(e) => {
                            error!("Error creating script in process added entities: {:?}", e);
                        }
                    };
                }
                for removed_entity in shared.removed_entities.drain(..) {
                    self.entity_scripts.remove(&removed_entity);
                    self.script_pending_removal.remove(&removed_entity);
                }
                for (entity, scope_cache) in shared.set_scope_variables.drain() {
                    if let Some(game_node_script) = self.entity_scripts.get_mut(&entity) {
                        for (key, value) in scope_cache {
                            game_node_script.update_scope(key, value);
                        }
                    }
                }
            }
            for entity in self.processed_added_entities.drain(..) {
                if let Some(game_node_script) = self.entity_scripts.get_mut(&entity) {
                    game_node_script.call(GameNodeScriptFunction::Init, engine, ());
                }
            }
            if let Some(shared) = self.shared.try_borrow() {
                processing_done = shared.added_entities.is_empty();
            }
        }
    }

    pub fn apply_entity_update_s(
        entity_scripts: &mut HashMap<Entity, GameNodeScript>,
        shared: &mut ECSShared,
        physics: &mut RapierSimulation,
        entity_update: EntityUpdate,
        engine: &Engine,
    ) {
        let entity = entity_update.id;

        match entity_update.kind {
            EntityUpdateKind::ProgressBar(progress_bar_update) => {
                shared
                    .entities
                    .ui_progress_bar
                    .entry(entity)
                    .or_insert(ProgressBar::new())
                    .update(progress_bar_update);
            }
            EntityUpdateKind::DynamicRigidBodyTypeProps(props) => {
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    if let Some(mut dynamic_rigid_body_props) =
                        shared.entities.dynamic_rigid_body_props.get_mut(&entity)
                    {
                        dynamic_rigid_body_props.update(props.clone());
                    }
                    physics.set_dynamic_rigid_body_props(*rigid_body_handle, props);
                }
            }
            EntityUpdateKind::FadeInEffect(fade_in_effect, duration) => {
                shared
                    .entities
                    .render_fadein_effect
                    .insert(entity, (fade_in_effect, duration));
            }
            EntityUpdateKind::FadeOutEffect(fade_out_effect, duration) => {
                shared
                    .entities
                    .render_fadeout_effect
                    .insert(entity, (fade_out_effect, duration));
            }
            EntityUpdateKind::Layer(layer) => {
                shared.entities.render_layer.insert(entity, layer);
            }
            EntityUpdateKind::InstancePath(_) => {
                error!("Instances should not exist on themselves inside ECS for now!");
            }
            EntityUpdateKind::Tags(tags) => {
                shared.entities.game_node_tags.insert(entity, tags);
            }
            EntityUpdateKind::Collider(collider) => {
                shared.entities.collider.insert(entity, collider.clone());
                if let Some(collider_handle) = shared.entities.collider_handle.get(&entity).cloned()
                {
                    if let Some(parent) =
                        shared
                            .entities
                            .game_node_parent
                            .get(&entity)
                            .and_then(|parent| {
                                if shared.entities.rigid_body_handle.contains_key(parent) {
                                    Some(*parent)
                                } else {
                                    None
                                }
                            })
                    {
                        physics.remove_collider(collider_handle);
                        shared.entities.collider_handle.remove(&entity);
                        shared.removed_colliders.push(collider_handle);
                        ECS::attach_collider_to_its_entity(&parent, &entity, shared, physics);
                    }
                }
            }
            EntityUpdateKind::Transform(transform) => {
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(transform.position.0, transform.position.1),
                        transform.rotation,
                        *rigid_body_handle,
                    );
                } else {
                    shared.entities.transforms.insert(entity, transform);
                }
            }
            EntityUpdateKind::KinematicCharacterControllerProps(props) => {
                if let Some(kinematic_character) =
                    shared.entities.kinematic_character.get_mut(&entity)
                {
                    kinematic_character.props = props.clone();
                    kinematic_character.controller =
                        RapierSimulation::create_kinematic_character_controller(&props);
                }
            }
            EntityUpdateKind::PositionRotation((x, y, r)) => {
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    physics.set_translation_and_rotation_for_rigid_body(
                        Vector::new(x, y),
                        r,
                        *rigid_body_handle,
                    );
                } else if let Some(transform) = shared.entities.transforms.get_mut(&entity) {
                    transform.position = (x, y);
                    transform.rotation = r;
                }
            }
            EntityUpdateKind::Name(name) => {
                shared.entities.game_node_name.insert(entity, name);
            }
            EntityUpdateKind::ScriptPath(script_path_option) => match script_path_option {
                Some(script_path) => match GameNodeScript::new(entity, engine, script_path) {
                    Ok(game_node_script) => {
                        entity_scripts.insert(entity, game_node_script);
                    }
                    Err(e) => {
                        error!("Error creating script in apply entity update: {:?}", e);
                    }
                },
                None => {
                    entity_scripts.remove(&entity);
                }
            },
            EntityUpdateKind::RigidBodyType(rigid_body_type) => {
                shared
                    .entities
                    .rigid_body_type
                    .insert(entity, rigid_body_type.clone());
                if let Some(rigid_body_handle) =
                    shared.entities.rigid_body_handle.get(&entity_update.id)
                {
                    match rigid_body_type {
                        RigidBodyType::KinematicPositionBased
                        | RigidBodyType::KinematicVelocityBased => {
                            if let Some(kinematic_character) =
                                shared.entities.kinematic_character.get_mut(&entity)
                            {
                                kinematic_character.props =
                                    KinematicCharacterControllerProps::new();
                                kinematic_character.controller =
                                    RapierSimulation::create_kinematic_character_controller(
                                        &kinematic_character.props,
                                    );
                            }
                            shared.entities.dynamic_rigid_body_props.remove(&entity);
                        }
                        RigidBodyType::Dynamic => {
                            shared.entities.kinematic_character.remove(&entity);
                            shared
                                .entities
                                .dynamic_rigid_body_props
                                .insert(entity, DynamicRigidBodyProps::new());
                        }
                        RigidBodyType::Fixed => {
                            shared.entities.kinematic_character.remove(&entity);
                            shared.entities.dynamic_rigid_body_props.remove(&entity);
                        }
                    }
                    physics.remove_rigid_body(*rigid_body_handle);
                    shared.entities.rigid_body_handle.remove(&entity);
                    let transform = shared
                        .entities
                        .transforms
                        .get(&entity)
                        .cloned()
                        .unwrap_or_default();
                    ECS::add_rigid_body_for_entity(
                        &entity,
                        &rigid_body_type,
                        &transform,
                        shared,
                        physics,
                    );
                    ECS::attach_colliders_to_entity(&entity, shared, physics);
                }
            }
            EntityUpdateKind::TextRender(text_render) => {
                shared.entities.text_render.insert(entity, text_render);
            }
            EntityUpdateKind::Gid(gid) => {
                shared.entities.render_gid.insert(entity, gid);
            }
            EntityUpdateKind::SpriteTilesetResource(resource_path) => {
                shared
                    .entities
                    .render_gid_tileset_path
                    .insert(entity, resource_path);
            }
            EntityUpdateKind::AnimatedSpriteResource(resource_path) => {
                Self::add_character_animation(shared, entity, &resource_path);
            }
            EntityUpdateKind::RenderKind(render_kind) => match render_kind {
                RenderKind::Text(text_render) => {
                    shared.entities.text_render.insert(entity, text_render);
                    shared
                        .entities
                        .render_kind
                        .insert(entity, RenderKindClean::Text);
                }
                RenderKind::AnimatedSprite(resource_path, _) => {
                    shared.entities.render_gid_tileset_path.remove(&entity);
                    Self::add_character_animation(shared, entity, &resource_path);
                }
                RenderKind::Sprite(resource_path, gid) => {
                    shared.entities.character_animation.remove(&entity);
                    shared
                        .entities
                        .render_kind
                        .insert(entity, RenderKindClean::Sprite);
                    shared.entities.render_gid.insert(entity, gid);
                    shared
                        .entities
                        .render_gid_tileset_path
                        .insert(entity, resource_path.clone());
                }
                RenderKind::ProgressBar(progress_bar) => {
                    shared.entities.ui_progress_bar.insert(entity, progress_bar);
                    shared
                        .entities
                        .render_kind
                        .insert(entity, RenderKindClean::ProgressBar);
                }
            },
            EntityUpdateKind::UpdateScriptScope(scope_key, scope_value) => {
                if let Some(game_node_script) = entity_scripts.get_mut(&entity) {
                    game_node_script.update_scope(scope_key, scope_value);
                }
            }
            EntityUpdateKind::SetScriptScope(scope_cache) => {
                if let Some(game_node_script) = entity_scripts.get_mut(&entity) {
                    for (key, value) in scope_cache {
                        game_node_script.update_scope(key, value);
                    }
                }
            }
        }
    }
}

impl ECSShared {
    pub fn get_parent_entity(&self, entity: &Entity) -> Option<Entity> {
        self.entities.game_node_parent.get(entity).cloned()
    }

    pub fn get_instance_root_entity(&self, entity: &Entity) -> Option<&Entity> {
        if let Some(Node2DKindClean::Instance) = self.entities.node_2d_kind.get(entity) {
            return self
                .entities
                .game_node_children
                .get(entity)
                .and_then(|children| children.first());
        }
        None
    }

    pub fn create_timer(&mut self, duration: f64) -> TimerId {
        let timer_id = self.timer_counter;
        self.timer_counter += 1;
        self.timer_map.insert(timer_id, Timer::new(duration));
        timer_id
    }

    pub fn create_tween(&mut self, time: f64, initial_value: f64, add_value: f64) -> TweenId {
        let tween_id = self.tween_counter;
        self.tween_counter += 1;
        self.tween_map
            .insert(tween_id, Tween::new(time, initial_value, add_value));
        tween_id
    }

    pub fn remove_timer(&mut self, timer_id: &TimerId) {
        self.timer_map.remove(timer_id);
    }

    pub fn remove_tween(&mut self, tween_id: &TweenId) {
        self.tween_map.remove(tween_id);
    }
}
