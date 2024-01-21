use std::cell::OnceCell;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::core::blueprint::def::{GameMap, GameNodeKind, Module, ResourcePath, Tileset};

struct ResourceCache {
    tilesets: RwLock<HashMap<ResourcePath, Tileset>>,
    maps: RwLock<HashMap<ResourcePath, GameMap>>,
    scenes: RwLock<HashMap<ResourcePath, Vec<GameNodeKind>>>,
    modules: RwLock<HashMap<ResourcePath, Module>>
}

static RESOURCE_CACHE: OnceCell<ResourceCache> = OnceCell::new();

fn get_resource_cache() -> &'static ResourceCache {
    RESOURCE_CACHE.get_or_init(|| {
        ResourceCache {
            tilesets: RwLock::new(HashMap::new()),
            maps: RwLock::new(HashMap::new()),
            scenes: RwLock::new(HashMap::new()),
            modules: RwLock::new(HashMap::new())
        }
    })
}

fn init_resource_cache() {

}
