use std::collections::hash_map::ValuesMut;
use std::collections::HashMap;
use std::hash::Hash;

impl<T> ManagedMapEntry<T> {
    pub fn mark_for_deletion(&mut self) {
        self.delete = true;
    }
}

pub struct ManagedMap<E: Hash + Eq + Clone, T> {
    pub data: HashMap<E, ManagedMapEntry<T>>,
    delete_queue: Vec<E>,
}

impl<E: Hash + Eq + Clone, T> ManagedMap<E, T> {
    pub fn new() -> ManagedMap<E, T> {
        ManagedMap {
            data: HashMap::new(),
            delete_queue: Vec::new(),
        }
    }
    pub fn entries_mut(&mut self) -> ValuesMut<'_, E, ManagedMapEntry<T>> {
        self.data.values_mut()
    }
    pub fn insert(&mut self, id: E, data: T) {
        self.data.insert(
            id,
            ManagedMapEntry {
                data,
                delete: false,
            },
        );
    }
    pub fn delete<F: FnMut(T)>(&mut self, mut cb: F) {
        for (id, entry) in &self.data {
            if entry.delete {
                self.delete_queue.push(id.clone())
            }
        }

        for id in self.delete_queue.drain(..) {
            if let Some(item) = self.data.remove(&id) {
                cb(item.data);
            }
        }
    }
}

pub struct ManagedMapEntry<T> {
    delete: bool,
    pub data: T,
}
