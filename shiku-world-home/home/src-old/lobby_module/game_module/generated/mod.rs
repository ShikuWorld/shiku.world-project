#![allow(dead_code)]
#![allow(unused_variables)]

use crate::core::entity_manager::{ColliderEntityMap, EntityManager};
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::core::terrain_gen::condense_terrain_from_tiles;
use crate::core::tween::Tween;
use crate::resource_module::def::GuestId;
use crate::resource_module::map::def::{
    GeneralObject, LayerName, ObjectId, TerrainChunk, TiledMap,
};
use log::debug;

use crate::core::entity::def::{Entity, EntityId, RemoveEntity, ShowEntity, UpdateEntity};
use crate::core::entity::physics::{
    physical_shape_from_general_obj, Physical, PhysicalShape, PhysicsArea, PhysicsNone,
    PhysicsRigidBody, PhysicsStaticRigidBody, ShapeRect,
};
use crate::core::entity::render::{
    NoRender, RenderTypeText, RenderTypeTimer, ShowEffect, StaticImage,
};
use crate::core::entity::terrain::TerrainEntity;

use rapier2d::prelude::{ColliderHandle, Isometry, Real, Vector};

use serde::Deserialize;
use snowflake::SnowflakeIdBucket;
use std::collections::HashMap;
use std::iter::{Filter, Map};

#[derive(Debug, Deserialize, PartialEq)]
pub enum LobbyGameObject {
    Terrain,
    Guest,
    EnterArea,
    Rolling,
    ExitArea,
    GuestNameplate,
    Observer,
    Timer,
    Text,
}

#[derive(Debug)]
pub struct Guest;

#[derive(Debug)]
pub struct EnterArea {
    pub slot_id: String,
}

#[derive(Debug)]
pub struct Rolling {
    pub direction: String,
}

#[derive(Debug)]
pub struct ExitArea {
    pub info: EntityId,
    pub guests: u32,
    pub open: bool,
    pub slot_id: String,
}

#[derive(Debug)]
pub struct GuestNameplate;

#[derive(Debug)]
pub struct Observer;

#[derive(Debug)]
pub struct Timer;

#[derive(Debug)]
pub struct Text;

pub struct GuestVariant {
pub gid_moved: &'static str,
pub gid_move: &'static str,
pub gid_default: &'static str,
pub shape_default: PhysicalShape,
pub offset_from_center_x: Real, pub offset_from_center_y: Real,
}

impl GuestVariant {
                    pub fn get_offset_2d(&self) -> (Real, Real) {
                        (self.offset_from_center_x, self.offset_from_center_y)
                    }
                }

pub struct GuestVariants {
pub default: GuestVariant,
}

impl Guest {
pub const VARIANTS: GuestVariants = GuestVariants {
default: GuestVariant {
gid_moved: "483",
gid_move: "482",
gid_default: "481",
shape_default: PhysicalShape::ShapeRect(ShapeRect { offset_from_center_x: 2.0, offset_from_center_y: 9.5, width: 12.0, height: 9.0 }),
offset_from_center_x: 2.00, offset_from_center_y: 9.50,
},
};
}

impl GuestVariants {
pub fn get_variant(&self, variant: &String) -> &GuestVariant {
match variant.as_str() {
"default" => &self.default,
_ => &self.default,
}
}
}

pub type GuestEntity = Entity<Guest, PhysicsRigidBody, StaticImage>;
pub type EnterAreaEntity = Entity<EnterArea, PhysicsArea, NoRender>;
pub type RollingEntity = Entity<Rolling, PhysicsArea, NoRender>;
pub type ExitAreaEntity = Entity<ExitArea, PhysicsArea, NoRender>;
pub type GuestNameplateEntity = Entity<GuestNameplate, PhysicsNone, RenderTypeText>;
pub type ObserverEntity = Entity<Observer, PhysicsArea, StaticImage>;
pub type TimerEntity = Entity<Timer, PhysicsNone, RenderTypeTimer>;
pub type TextEntity = Entity<Text, PhysicsNone, RenderTypeText>;

impl GuestEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> Guest {
        Guest {

        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &GuestVariant,
        game_state: Guest,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GuestEntity {
let physics_body = PhysicsRigidBody::create(
                pos,
                &physics_instructions.shape_default,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage { width: None, height: None, tiled: false, layer, graphic_id, blending_mode: None, scale: (1.0, 1.0), offset_2d: physics_instructions.get_offset_2d() },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GuestEntity {
        let physics_instructions = Guest::VARIANTS.get_variant(&general_object.get_custom_prop_string("variant"));

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GuestEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl EnterAreaEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> EnterArea {
        EnterArea {
   slot_id: obj.get_custom_prop_string("slot_id"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: EnterArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> EnterAreaEntity {
let physics_body = PhysicsArea::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> EnterAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            EnterAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl RollingEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> Rolling {
        Rolling {
   direction: obj.get_custom_prop_string("direction"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Rolling,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> RollingEntity {
let physics_body = PhysicsArea::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> RollingEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            RollingEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl ExitAreaEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> ExitArea {
        ExitArea {
   info: obj.get_custom_prop_entity_id("info"),
   guests: obj.get_custom_prop_u_32("guests"),
   open: obj.get_custom_prop_bool("open"),
   slot_id: obj.get_custom_prop_string("slot_id"),
        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: ExitArea,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> ExitAreaEntity {
let physics_body = PhysicsArea::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: NoRender {},
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> ExitAreaEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            ExitAreaEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl GuestNameplateEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> GuestNameplate {
        GuestNameplate {

        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: GuestNameplate,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> GuestNameplateEntity {
let physics_body = PhysicsNone::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: RenderTypeText::from_general_object(&general_object, layer),
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> GuestNameplateEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            GuestNameplateEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl ObserverEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> Observer {
        Observer {

        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Observer,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> ObserverEntity {
let physics_body = PhysicsArea::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: StaticImage { width: None, height: None, tiled: false, layer, graphic_id, blending_mode: None, scale: (1.0, 1.0), offset_2d: physics_instructions.get_offset_2d() },
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> ObserverEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            ObserverEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl TimerEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> Timer {
        Timer {

        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Timer,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> TimerEntity {
let physics_body = PhysicsNone::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: RenderTypeTimer::from_general_object(&general_object, layer),
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> TimerEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            TimerEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

impl TextEntity {

    pub fn game_state_from_general_object(obj: &GeneralObject) -> Text {
        Text {

        }
    }

    pub fn new(
        entity_id: EntityId,
        pos: Vector<Real>,
        graphic_id: String,
        layer: LayerName,
        physics_instructions: &PhysicalShape,
        game_state: Text,
        general_object: Option<GeneralObject>,
        physics: &mut RapierSimulation,
    ) -> TextEntity {
let physics_body = PhysicsNone::create(
                pos,
                &physics_instructions,
                physics,
            );

        Entity {
            id: entity_id,
            isometry: physics_body.position(physics),
            physics: physics_body,
            render: RenderTypeText::from_general_object(&general_object, layer),
            game_state,
            is_render_dirty: false,
            is_position_dirty: false,
            general_object,
            parent_entity: None,
        }
    }

    pub fn new_from_general_object(
        entity_id: EntityId,
        general_object: &GeneralObject,
        layer_name: LayerName,
        physics: &mut RapierSimulation,
    ) -> TextEntity {
        let physics_instructions = &physical_shape_from_general_obj(general_object);

        Self::new(
            entity_id,
            Vector::new(general_object.x, general_object.y),
            general_object.graphic_id.clone(),
            layer_name,
            physics_instructions,
            TextEntity::game_state_from_general_object(general_object),
            Some(general_object.clone()),
            physics,
        )
    }

}

pub struct LobbyGameEntityManager {
    pub guest_map: HashMap<EntityId, GuestEntity>,
    pub enter_area_map: HashMap<EntityId, EnterAreaEntity>,
    pub rolling_map: HashMap<EntityId, RollingEntity>,
    pub exit_area_map: HashMap<EntityId, ExitAreaEntity>,
    pub guest_nameplate_map: HashMap<EntityId, GuestNameplateEntity>,
    pub observer_map: HashMap<EntityId, ObserverEntity>,
    pub timer_map: HashMap<EntityId, TimerEntity>,
    pub text_map: HashMap<EntityId, TextEntity>,
    entity_id_generator: SnowflakeIdBucket,
    pub terrain_map: HashMap<EntityId, TerrainEntity>,
    pub collider_entity_map: ColliderEntityMap<LobbyGameObject>,
    terrain_chunks: Vec<TerrainChunk>,
    guest_id_to_camera_entity_id_map: HashMap<GuestId, EntityId>,
    new_show_entities: Vec<ShowEntity>,
    new_remove_entities: Vec<RemoveEntity>,
    pub new_show_effects: Vec<ShowEffect>,
}

impl LobbyGameEntityManager {
    pub fn new() -> LobbyGameEntityManager {
        LobbyGameEntityManager {
            entity_id_generator: SnowflakeIdBucket::new(1, 2),
            collider_entity_map: ColliderEntityMap::new(),
     
            terrain_map: HashMap::new(),
            guest_map: HashMap::new(),
            enter_area_map: HashMap::new(),
            rolling_map: HashMap::new(),
            exit_area_map: HashMap::new(),
            guest_nameplate_map: HashMap::new(),
            observer_map: HashMap::new(),
            timer_map: HashMap::new(),
            text_map: HashMap::new(),
            terrain_chunks: Vec::new(),
            guest_id_to_camera_entity_id_map: HashMap::new(),
            new_show_entities: Vec::new(),
            new_remove_entities: Vec::new(),
            new_show_effects: Vec::new(),
        }
    }
       pub fn create_guest<F: FnMut(&mut GuestEntity)>(
        &mut self,
        game_state: Guest,
        pos: Vector<Real>,
        physics_instructions: &GuestVariant,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GuestEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG0,
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), LobbyGameObject::Guest),
            );
        }

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.guest_map.insert(entity_id.clone(), entity);
    
        entity_id
    }
    
    pub fn remove_guest(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<GuestEntity> {
        if let Some(entity) = self.guest_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

   pub fn create_guest_nameplate<F: FnMut(&mut GuestNameplateEntity)>(
        &mut self,
        game_state: GuestNameplate,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = GuestNameplateEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG0,
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), LobbyGameObject::GuestNameplate),
            );
        }

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.guest_nameplate_map.insert(entity_id.clone(), entity);
    
        entity_id
    }
    
    pub fn remove_guest_nameplate(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<GuestNameplateEntity> {
        if let Some(entity) = self.guest_nameplate_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

   pub fn create_observer<F: FnMut(&mut ObserverEntity)>(
        &mut self,
        game_state: Observer,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = ObserverEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG3,
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), LobbyGameObject::Observer),
            );
        }

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.observer_map.insert(entity_id.clone(), entity);
    
        entity_id
    }
    
    pub fn remove_observer(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<ObserverEntity> {
        if let Some(entity) = self.observer_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

   pub fn create_timer<F: FnMut(&mut TimerEntity)>(
        &mut self,
        game_state: Timer,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = TimerEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG3,
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), LobbyGameObject::Timer),
            );
        }

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.timer_map.insert(entity_id.clone(), entity);
    
        entity_id
    }
    
    pub fn remove_timer(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<TimerEntity> {
        if let Some(entity) = self.timer_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }

   pub fn create_text<F: FnMut(&mut TextEntity)>(
        &mut self,
        game_state: Text,
        pos: Vector<Real>,
        physics_instructions: &PhysicalShape,
        graphic_id: String,
        physics: &mut RapierSimulation,
        mut adjust_callback: F
    ) -> EntityId {
        let entity_id = self.entity_id_generator.get_id().to_string();
        let mut entity = TextEntity::new(
            entity_id.clone(),
            pos,
            graphic_id,
            LayerName::FG10,
            physics_instructions,
            game_state,
            None,
            physics,
        );
    
        for collider_handle in entity.physics.get_all_collider_handles() {
            self.collider_entity_map.insert(
                collider_handle,
                (entity_id.clone(), LobbyGameObject::Text),
            );
        }

        adjust_callback(&mut entity);
    
        self.new_show_entities.push(entity.show_entity());
    
        self.text_map.insert(entity_id.clone(), entity);
    
        entity_id
    }
    
    pub fn remove_text(&mut self, entity_id: &EntityId, physics: &mut RapierSimulation) -> Option<TextEntity> {
        if let Some(entity) = self.text_map.remove(entity_id) {
            self.new_remove_entities.push(entity.remove_entity());
            entity.physics.remove(physics);

            for collider_handle in entity.physics.get_all_collider_handles() {
                self.collider_entity_map.remove(&collider_handle);
            }

            return Some(entity);
        }

        None
    }
}

impl EntityManager for LobbyGameEntityManager {
    fn create_initial(&mut self, map: &TiledMap, physics: &mut RapierSimulation) {
        for layer in &map.layers {
            self.terrain_chunks.extend(layer.terrain_chunks.clone());

            if let LayerName::Terrain = &layer.name {
                let chunks = condense_terrain_from_tiles(layer);
                for chunk in chunks {
                    let (body_handle, collider_handle) =
                        TerrainEntity::create_terrain_collider(&chunk, physics);

                    let terrain_entity = TerrainEntity::new_entity(
                        self.entity_id_generator.get_id().to_string(),
                        Isometry::new(Vector::new(0.0, 0.0), 0.0),
                        body_handle,
                        collider_handle,
                    );
                    self.collider_entity_map.insert(
                        collider_handle,
                        (terrain_entity.id.clone(), LobbyGameObject::Terrain),
                    );
                    self.terrain_map
                        .insert(terrain_entity.id.clone(), terrain_entity);
                }
            }
        }

        let mut reference_map = HashMap::<ObjectId, EntityId>::new();

        for group in &map.object_groups {
            for object in &group.objects {
                if let Ok(kind) = serde_json::from_str(object.kind.as_str()) {
                    let entity_id = self.entity_id_generator.get_id().to_string();
                    reference_map.insert(object.id.clone(), entity_id.clone());

                    match kind {
                     LobbyGameObject::Guest => {
                        let entity = GuestEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::Guest),
                            );
                        }

                        self.guest_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::EnterArea => {
                        let entity = EnterAreaEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::EnterArea),
                            );
                        }

                        self.enter_area_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::Rolling => {
                        let entity = RollingEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::Rolling),
                            );
                        }

                        self.rolling_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::ExitArea => {
                        let entity = ExitAreaEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::ExitArea),
                            );
                        }

                        self.exit_area_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::GuestNameplate => {
                        let entity = GuestNameplateEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::GuestNameplate),
                            );
                        }

                        self.guest_nameplate_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::Observer => {
                        let entity = ObserverEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::Observer),
                            );
                        }

                        self.observer_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::Timer => {
                        let entity = TimerEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::Timer),
                            );
                        }

                        self.timer_map.insert(entity_id, entity);
                    }
                     LobbyGameObject::Text => {
                        let entity = TextEntity::new_from_general_object(
                            entity_id.clone(),
                            object,
                            group.layer_name.clone(),
                            physics,
                        );
                    
                        for collider_handle in entity.physics.get_all_collider_handles() {
                            self.collider_entity_map.insert(
                                collider_handle,
                                (entity_id.clone(), LobbyGameObject::Text),
                            );
                        }

                        self.text_map.insert(entity_id, entity);
                    }
                        kind => {
                            debug!("Not generated right now {:?}", kind);
                        }
                    }
                }
            }
        }

       for entity in self.exit_area_map.values_mut() {
            if let Some(entity_id) = reference_map.get_mut(&entity.game_state.info) {
                entity.game_state.info = entity_id.clone();
            }
        }
    }

    fn update_entity_positions(&mut self, physics: &mut RapierSimulation) {
for entity in self.guest_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.enter_area_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.rolling_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.exit_area_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.guest_nameplate_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.observer_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.timer_map.values_mut() {
            entity.update_isometry(physics);
        }
for entity in self.text_map.values_mut() {
            entity.update_isometry(physics);
        }
    }

    fn set_camera_entity_for_guest(&mut self, guest_id: GuestId, entity_id: EntityId) {
        self.guest_id_to_camera_entity_id_map
            .insert(guest_id, entity_id);
    }

    fn get_current_camera_entity_for_guest(&self, guest_id: &GuestId) -> EntityId {
        self.guest_id_to_camera_entity_id_map
            .get(guest_id)
            .unwrap_or(&String::new())
            .clone()
    }

    fn get_all_terrain_chunks(&mut self) -> Vec<TerrainChunk> {
        self.terrain_chunks.clone()
    }

    fn get_all_show_entities(&mut self) -> Vec<ShowEntity> {
        let mut show_entities = Vec::new();
        for entity in self.guest_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.guest_nameplate_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.observer_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.timer_map.values() {
            show_entities.push(entity.show_entity());
        }
        for entity in self.text_map.values() {
            show_entities.push(entity.show_entity());
        }
        show_entities
    }

    fn get_all_entity_updates(&mut self) -> Vec<UpdateEntity> {
        let mut entity_updates = Vec::new();
        for entity in self.guest_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.guest_nameplate_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.observer_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.timer_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        for entity in self.text_map.values_mut() {
            if entity.is_render_dirty {
                entity_updates.push(entity.update_entity());
                entity.is_render_dirty = false;
            }
        }
        entity_updates
    }

    fn get_all_entity_position_updates(&mut self) -> Vec<(EntityId, Real, Real, Real)> {
        let mut position_updates = Vec::new();
        for entity in self.guest_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.guest_nameplate_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.observer_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.timer_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        for entity in self.text_map.values_mut() {
            if entity.is_position_dirty {
                position_updates.push(entity.position_update());
                entity.is_position_dirty = false;
            }
        }
        position_updates
    }

    fn drain_new_show_effects(&mut self) -> Vec<ShowEffect> {
        self.new_show_effects.drain(..).collect()
    }

    fn drain_new_show_entities(&mut self) -> Vec<ShowEntity> {
        self.new_show_entities.drain(..).collect()
    }

    fn drain_new_remove_entities(&mut self) -> Vec<RemoveEntity> {
        self.new_remove_entities.drain(..).collect()
    }
}