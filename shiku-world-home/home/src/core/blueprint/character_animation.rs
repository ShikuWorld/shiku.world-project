use crate::core::blueprint::def::Gid;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use ts_rs::TS;

pub type StateName = u32;
pub type TransitionName = u32;

#[derive(TS, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[ts(export, export_to = "blueprints/")]
pub enum CharacterDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterAnimation {
    pub id: String,
    pub name: String,
    pub resource_path: String,
    pub tileset_resource: String,
    pub current_direction: CharacterDirection,
    pub current_state: StateName,
    pub current_gid_inside_tile: Gid,
    pub states: HashMap<StateName, CharacterAnimationState>,
    pub transitions: HashMap<TransitionName, HashMap<StateName, StateName>>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterAnimationState {
    name: String,
    frames: Vec<CharacterAnimationFrame>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterAnimationFrame {
    duration_in_ms: Real,
    gid_map: HashMap<CharacterDirection, Gid>,
}

impl CharacterAnimationFrame {
    pub fn new(
        duration_in_ms: Real,
        gid_map: HashMap<CharacterDirection, Gid>,
    ) -> CharacterAnimationFrame {
        CharacterAnimationFrame {
            duration_in_ms,
            gid_map,
        }
    }

    pub fn get_gid(&self, qualifier: &CharacterDirection) -> Gid {
        *self.gid_map.get(qualifier).unwrap_or(&0)
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    current_frame_time_in_ms: Real,
    current_total_time_in_ms: Real,
    animation_total_duration: Real,
    frames: Vec<CharacterAnimationFrame>,
    current_frame_index: usize,
    running: bool,
    pub done: bool,
}

impl Animation {
    pub fn new(frames: Vec<CharacterAnimationFrame>) -> Animation {
        let animation_total_duration: Real = frames.iter().map(|f| f.duration_in_ms).sum();

        Animation {
            current_frame_time_in_ms: 0.0,
            current_total_time_in_ms: 0.0,
            current_frame_index: 0,
            animation_total_duration,
            frames,
            running: false,
            done: false,
        }
    }

    pub fn progress(&self) -> Real {
        if self.done {
            return 1.0;
        }

        self.current_total_time_in_ms / self.animation_total_duration
    }

    pub fn start(&mut self) {
        self.reset();
        self.running = true;
    }

    pub fn reset(&mut self) {
        self.done = false;
        self.running = false;
        self.current_frame_index = 0;
        self.current_frame_time_in_ms = 0.0;
        self.current_total_time_in_ms = 0.0;
    }

    pub fn run(&mut self, update_time: Real) {
        if self.done {
            return;
        }

        self.current_frame_time_in_ms += update_time;
        self.current_total_time_in_ms += update_time;

        if let Some(current_frame) = self.frames.get(self.current_frame_index) {
            if self.current_frame_time_in_ms > current_frame.duration_in_ms {
                if self.frames.get(self.current_frame_index + 1).is_some() {
                    self.current_frame_time_in_ms -= current_frame.duration_in_ms;
                    self.current_frame_index += 1;
                } else {
                    self.done = true;
                    self.running = false;
                }
            }
        }
    }

    pub fn get_current_gid(&self, current_qualifier: &CharacterDirection) -> Gid {
        if let Some(animation_frame) = self.frames.get(self.current_frame_index) {
            return animation_frame.get_gid(current_qualifier);
        }

        0
    }
}
