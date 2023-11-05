use std::collections::{HashMap, HashSet};

use rapier2d::prelude::{ColliderHandle, Real};

use crate::core::entity::def::{EntityId, RemoveEntity, ShowEntity, UpdateEntity};
use crate::core::entity::render::ShowEffect;
use crate::core::rapier_simulation::def::RapierSimulation;
use crate::resource_module::def::ActorId;
use crate::resource_module::map::def::{TerrainChunk, TiledMap};

pub type HideEntityMap = HashMap<ActorId, HashSet<EntityId>>;

pub trait EntityVisibility {
    fn hide_entity(&mut self, guest_id: &ActorId, entity_id: &EntityId);
    fn show_entity(&mut self, guest_id: &ActorId, entity_id: &EntityId);
    fn entity_hidden(&mut self, guest_id: &ActorId, entity_id: &EntityId) -> bool;
}

impl EntityVisibility for HideEntityMap {
    fn hide_entity(&mut self, guest_id: &ActorId, entity_id: &EntityId) {
        self.entry(*guest_id)
            .or_insert_with(HashSet::new)
            .insert(entity_id.clone());
    }

    fn show_entity(&mut self, guest_id: &ActorId, entity_id: &EntityId) {
        self.entry(*guest_id)
            .or_insert_with(HashSet::new)
            .remove(entity_id);
    }

    fn entity_hidden(&mut self, guest_id: &ActorId, entity_id: &EntityId) -> bool {
        self.entry(*guest_id)
            .or_insert_with(HashSet::new)
            .contains(entity_id)
    }
}

pub trait EntityManager {
    fn create_initial(&mut self, map: &TiledMap, physics: &mut RapierSimulation);
    fn update_entity_positions(&mut self, physics: &mut RapierSimulation);

    fn set_camera_entity_for_guest(&mut self, guest_id: ActorId, entity_id: EntityId);
    fn get_current_camera_entity_for_guest(&self, guest_id: &ActorId) -> EntityId;
    fn get_all_terrain_chunks(&mut self) -> Vec<TerrainChunk>;
    fn get_all_show_entities(&mut self) -> Vec<ShowEntity>;
    fn get_all_entity_updates(&mut self) -> Vec<UpdateEntity>;
    fn get_all_entity_position_updates(&mut self) -> Vec<(EntityId, Real, Real, Real)>;

    fn drain_new_show_effects(&mut self) -> Vec<ShowEffect>;
    fn drain_new_show_entities(&mut self) -> Vec<ShowEntity>;
    fn drain_new_remove_entities(&mut self) -> Vec<RemoveEntity>;
}

pub struct ColliderEntityMap<E: PartialEq> {
    map: HashMap<ColliderHandle, (EntityId, E)>,
}

impl<E: PartialEq> ColliderEntityMap<E> {
    pub fn new() -> ColliderEntityMap<E> {
        ColliderEntityMap {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, collider_handle: ColliderHandle, entry: (EntityId, E)) {
        self.map.insert(collider_handle, entry);
    }

    pub fn remove(&mut self, collider_handle: &ColliderHandle) -> Option<(EntityId, E)> {
        self.map.remove(collider_handle)
    }

    pub fn get(&self, collider_handle: &ColliderHandle) -> Option<&(EntityId, E)> {
        self.map.get(collider_handle)
    }

    pub fn has(&self, collider_handle: &ColliderHandle, of_kind: E) -> bool {
        if let Some((_entity_id, kind)) = self.map.get(collider_handle) {
            return of_kind == *kind;
        }
        return false;
    }

    pub fn entities_from_colliders<T, F: FnMut(&mut T)>(
        &self,
        intersecting_colliders: &Vec<ColliderHandle>,
        entity_map: &mut HashMap<EntityId, T>,
        mut callback: F,
    ) {
        for collider_handle in intersecting_colliders {
            if let Some((entity_id, _object_type)) = self.map.get(&collider_handle) {
                if let Some(entity) = entity_map.get_mut(entity_id) {
                    callback(entity);
                }
            }
        }
    }
}
