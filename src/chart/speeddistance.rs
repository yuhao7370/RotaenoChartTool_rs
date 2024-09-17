// speed.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SpeedDistance {
    pub time: f32,
    pub speed: f32,
    pub smooth: i32,
    pub distance: f32,
}

impl SpeedDistance {
    pub fn new(time: f32, speed: f32, smooth: i32, distance: f32) -> Self {
        Self { time, speed, smooth, distance }
    }
}