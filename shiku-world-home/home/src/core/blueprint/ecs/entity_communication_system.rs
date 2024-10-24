use crate::core::blueprint::ecs::def::Entity;
use rhai::{Dynamic, ImmutableString};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct EntityCommunicationSystem {
    pub event_queue: Vec<(Entity, ImmutableString, Dynamic)>,
    pub entity_to_subscribers_map: HashMap<Entity, HashSet<Entity>>,
    pub subscriber_to_entities_map: HashMap<Entity, HashSet<Entity>>,
    pub last_value_cache: HashMap<Entity, HashMap<ImmutableString, Dynamic>>,
}

impl EntityCommunicationSystem {
    pub fn new() -> Self {
        Self {
            event_queue: Vec::new(),
            entity_to_subscribers_map: HashMap::new(),
            subscriber_to_entities_map: HashMap::new(),
            last_value_cache: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, entity: Entity, subscriber: Entity) {
        self.entity_to_subscribers_map
            .entry(entity)
            .or_insert_with(HashSet::new)
            .insert(subscriber);

        self.subscriber_to_entities_map
            .entry(subscriber)
            .or_insert_with(HashSet::new)
            .insert(entity);
    }

    pub fn unsubscribe(&mut self, entity: Entity, subscriber: Entity) {
        if let Some(subscribers) = self.entity_to_subscribers_map.get_mut(&entity) {
            subscribers.remove(&subscriber);
        }

        if let Some(entities) = self.subscriber_to_entities_map.get_mut(&subscriber) {
            entities.remove(&entity);
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(subscribers) = self.entity_to_subscribers_map.remove(&entity) {
            for subscriber in subscribers {
                if let Some(entities) = self.subscriber_to_entities_map.get_mut(&subscriber) {
                    entities.remove(&entity);
                }
            }
        }
    }

    pub fn publish(&mut self, entity: Entity, event: ImmutableString, data: Dynamic) {
        self.last_value_cache
            .entry(entity)
            .or_default()
            .insert(event.clone(), data.clone());
        self.event_queue.push((entity, event, data));
    }

    pub fn get_last_cached_value(&self, entity: Entity, event: &str) -> Dynamic {
        self.last_value_cache
            .get(&entity)
            .and_then(|cache| cache.get(event).cloned())
            .unwrap_or(Dynamic::UNIT)
    }
}
