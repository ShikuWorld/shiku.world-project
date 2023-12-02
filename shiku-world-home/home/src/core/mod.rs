use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;

use flume::Sender;
use log::{debug, error};
use rapier2d::prelude::Real;

pub mod animation;
pub mod basic_game_module;
pub mod blending_mode;
pub mod entity;
pub mod entity_manager;
pub mod entity_manager_generation;
pub mod game_module_communication;
pub mod managed_map;
pub mod medium_data_storage;
pub mod module_system;

pub mod blueprint;
pub mod guest;
pub mod module;
pub mod rapier_simulation;
pub mod resource_json_generation;
pub mod ring;
pub mod terrain_gen;
pub mod tween;

pub type Snowflake = i64;

pub const LOGGED_IN_TODAY_DELAY_IN_HOURS: i64 = 16;
pub const TARGET_FPS: Real = 60.0;
pub const TARGET_FRAME_DURATION: Real = 1000.0 / 60.0;

pub struct LazyHashmapSet<K: Eq + Hash, T: Eq + Hash> {
    data: HashMap<K, HashSet<T>>,
}

impl<K: Eq + Hash, T: Eq + Hash> LazyHashmapSet<K, T> {
    pub fn new() -> LazyHashmapSet<K, T> {
        LazyHashmapSet {
            data: HashMap::new(),
        }
    }
    pub fn init(&mut self, key: K) {
        self.data.insert(key, HashSet::new());
    }
    pub fn remove(&mut self, key: &K) -> Option<HashSet<T>> {
        self.data.remove(key)
    }
    pub fn remove_entry(&mut self, key: &K, value: &T) -> bool {
        if let Some(data) = self.data.get_mut(key) {
            return data.remove(value);
        }
        false
    }
    pub fn insert_entry(&mut self, key: &K, value: T) {
        if let Some(data) = self.data.get_mut(key) {
            data.insert(value);
        }
    }

    pub fn filter<F>(&mut self, key: K, mut callback: F)
    where
        F: FnMut(&T) -> bool,
    {
        if let Some(vec) = self.data.get_mut(&key) {
            vec.retain(callback);
        }
    }

    pub fn len(&mut self, key: &K) -> usize {
        if let Some(data) = self.data.get_mut(key) {
            return data.len();
        }
        0
    }
}

pub fn safe_unwrap<T, E>(option: Option<T>, err: E) -> Result<T, E> {
    match option {
        Some(val) => Ok(val),
        None => Err(err),
    }
}

pub fn safe_unwrap_ref<T, E>(option: &Option<T>, err: E) -> Result<&T, E> {
    match option {
        Some(val) => Ok(val),
        None => Err(err),
    }
}

pub fn get_out_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        let mut path = exe_path;
        path.pop();
        path.join("out")
    } else {
        PathBuf::from(Path::new("./out"))
    }
}

pub fn send_and_log_error<T>(sender: &mut Sender<T>, data: T) {
    if let Err(err) = sender.send(data) {
        error!("{:?}", err);
    }
}

pub fn send_and_log_error_consume<T>(sender: Sender<T>, data: T) {
    if let Err(err) = sender.send(data) {
        error!("{:?}", err);
    }
}

pub fn log_result_error<O, F: Debug>(result: Result<O, F>) {
    if let Err(err) = result {
        debug!("{:?}", err);
    }
}

pub fn fix_intellij_error_bug<T: fmt::Debug + fmt::Display>(error: &T) -> impl fmt::Display + '_ {
    struct DisplayWrapper<'a, T: fmt::Debug + fmt::Display>(&'a T);

    impl<'a, T: fmt::Debug + fmt::Display> fmt::Display for DisplayWrapper<'a, T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt::Display::fmt(self.0, f)
        }
    }

    DisplayWrapper(error)
}
