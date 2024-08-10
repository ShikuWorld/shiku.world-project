use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tween {
    pub repeat: bool,
    time: f64,
    pub initial_value: f64,
    pub add_value: f64,
    current_time: f64,
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

    pub fn update(&mut self, time_update: f64) {
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

    pub fn progress(&mut self) -> f64 {
        self.current_time / self.time
    }

    pub fn current_value(&mut self) -> f64 {
        self.initial_value + (self.progress() * self.add_value)
    }

    pub fn set_time(&mut self, time: f64) {
        self.time = if time == 0.0 { 1.0 } else { time };
    }

    pub fn new(time: f64, initial_value: f64, add_value: f64) -> Tween {
        Tween {
            repeat: false,
            time,
            initial_value,
            add_value,
            current_time: 0.0,
            running: false,
            done: false,
            backwards: false,
        }
    }
}
