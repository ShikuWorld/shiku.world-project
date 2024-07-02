use crate::core::blueprint::character_animation::{
    CharacterAnimation as CharacterAnimationBlueprint, CharacterAnimationFrame, CharacterDirection,
    StateId,
};
use crate::core::blueprint::def::Gid;
use rapier2d::math::Real;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterAnimation {
    pub blueprint: CharacterAnimationBlueprint,
    pub current_direction: CharacterDirection,
    pub current_state: StateId,
    pub current_gid: Gid,
    pub states: HashMap<StateId, Animation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Animation {
    current_frame_time_in_ms: Real,
    current_total_time_in_ms: Real,
    animation_total_duration: Real,
    frames: Vec<CharacterAnimationFrame>,
    current_frame_index: usize,
    running: bool,
    pub done: bool,
}

impl CharacterAnimation {
    pub fn run_current_animation(&mut self, update_time: Real) {
        if let Some(animation) = self.states.get_mut(&self.current_state) {
            animation.run(update_time);
            self.current_gid = animation.get_current_gid(&self.current_direction);
        }
    }

    pub fn go_to_state(&mut self, state_id: StateId) {
        if self.current_state == state_id {
            return;
        }

        if let Some(animation) = self.states.get_mut(&state_id) {
            self.current_state = state_id;
            self.current_gid = animation.get_current_gid(&self.current_direction);
            animation.start();
        }
    }
}

impl From<CharacterAnimationBlueprint> for CharacterAnimation {
    fn from(blueprint: CharacterAnimationBlueprint) -> Self {
        let mut states: HashMap<StateId, Animation> = blueprint
            .states
            .iter()
            .map(|(state_id, state)| {
                let animation = Animation::new(state.frames.clone());
                (*state_id, animation)
            })
            .collect();
        let mut start_gid = 0;
        if let Some(start_animation) = states.get_mut(&blueprint.start_state) {
            start_gid = start_animation.get_current_gid(&blueprint.start_direction);
            start_animation.start();
        }

        CharacterAnimation {
            current_direction: blueprint.start_direction.clone(),
            current_state: blueprint.start_state,
            blueprint,
            current_gid: start_gid,
            states,
        }
    }
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
