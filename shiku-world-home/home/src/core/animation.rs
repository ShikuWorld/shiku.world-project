use rapier2d::prelude::Real;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct AnimationFrame<Q: Clone + Eq + Hash> {
    duration_in_ms: Real,
    gid_map: HashMap<Q, &'static str>,
}

impl<Q: Clone + Eq + Hash> AnimationFrame<Q> {
    pub fn new(duration_in_ms: Real, gid_map: HashMap<Q, &'static str>) -> AnimationFrame<Q> {
        AnimationFrame {
            duration_in_ms,
            gid_map,
        }
    }

    pub fn get_gid(&self, qualifier: &Q) -> &'static str {
        self.gid_map.get(qualifier).unwrap_or(&"0")
    }
}

#[derive(Clone)]
pub struct Animation<Q: Clone + Eq + Hash> {
    current_frame_time_in_ms: Real,
    current_total_time_in_ms: Real,
    animation_total_duration: Real,
    frames: Vec<AnimationFrame<Q>>,
    current_frame_index: usize,
    running: bool,
    pub done: bool,
}

impl<Q: Clone + Eq + Hash> Animation<Q> {
    pub fn new(frames: Vec<AnimationFrame<Q>>) -> Animation<Q> {
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

    pub fn get_current_gid(&self, current_qualifier: &Q) -> &'static str {
        if let Some(animation_frame) = self.frames.get(self.current_frame_index) {
            return animation_frame.get_gid(current_qualifier);
        }

        "1"
    }
}
