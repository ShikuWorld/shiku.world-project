#[derive(Debug, Clone, PartialEq)]
pub struct Timer {
    duration: f64,
    current_time: f64,
    running: bool,
}

impl Timer {
    pub fn new(duration: f64) -> Timer {
        Timer {
            duration,
            current_time: 0.0,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
        self.current_time = 0.0;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self, delta_time: f64) {
        if !self.running {
            return;
        }

        self.current_time += delta_time;

        if self.current_time >= self.duration {
            self.current_time = self.duration;
            self.running = false;
        }
    }

    pub fn progress(&self) -> f64 {
        if self.duration == 0.0 {
            1.0
        } else {
            (self.current_time / self.duration).clamp(0.0, 1.0)
        }
    }

    pub fn is_finished(&self) -> bool {
        !self.running && self.current_time >= self.duration
    }

    pub fn set_duration(&mut self, duration: f64) {
        self.duration = duration.max(0.001);
    }
}
