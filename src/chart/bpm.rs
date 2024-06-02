// bpm.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BPM {
    pub time: f32,
    pub bpm: f32,
}

impl BPM {
    pub fn new(time: f32, bpm: f32) -> Self {
        Self { time, bpm }
    }

    pub fn real_time(&self) -> f32 {
        self.time / (31.25 * 32.0)
    }
}