use crate::core::blueprint::def::Gid;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use ts_rs::TS;

pub type StateId = u32;
pub type TransitionId = u32;

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
    pub start_direction: CharacterDirection,
    pub start_state: StateId,
    pub states: HashMap<StateId, CharacterAnimationState>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterTransitionFunctions {
    name: String,
    possible_transitions: HashMap<StateId, StateId>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterAnimationState {
    name: String,
    pub(crate) frames: Vec<CharacterAnimationFrame>,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "blueprints/")]
pub struct CharacterAnimationFrame {
    pub(crate) duration_in_ms: Real,
    pub(crate) gid_map: HashMap<CharacterDirection, Gid>,
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
