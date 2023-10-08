use rapier2d::math::Real;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TweenProp {
    PositionX,
    PositionY,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tween {
    pub repeat: bool,
    time: Real,
    pub initial_value: Real,
    pub add_value: Real,
    pub property: TweenProp,
    current_time: Real,
    running: bool,
    backwards: bool,
    done: bool,
}

impl Tween {
    pub fn start(&mut self) {
        self.running = true;
        self.done = false;
        self.current_time = 0.0;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self, time_update: Real) {
        if self.done {
            return;
        }

        self.current_time += if self.backwards {
            -time_update
        } else {
            time_update
        };

        if self.backwards {
            if self.current_time <= 0.0 {
                self.current_time = 0.0;
                if self.repeat {
                    self.backwards = !self.backwards;
                } else {
                    self.running = false;
                    self.done = true;
                }
            }
        } else if self.current_time > self.time {
            self.current_time = self.time;
            if self.repeat {
                self.backwards = !self.backwards;
            } else {
                self.running = false;
                self.done = true;
            }
        }
    }

    pub fn current_value(&mut self) -> Real {
        let progress = self.current_time / self.time;

        self.initial_value + (progress * self.add_value)
    }

    pub fn set_time(&mut self, time: Real) {
        self.time = if time == 0.0 { 1.0 } else { time };
    }

    pub fn new() -> Tween {
        Tween {
            repeat: false,
            time: 1.0,
            initial_value: 0.0,
            add_value: 0.0,
            current_time: 0.0,
            running: false,
            property: TweenProp::PositionY,
            done: false,
            backwards: false,
        }
    }
}
