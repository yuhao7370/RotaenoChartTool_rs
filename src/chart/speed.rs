// speed.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Speed {
    pub time: f32,
    pub speed: f32,
    pub smooth: i32,
}

impl Speed {
    pub fn new(time: f32, speed: f32, smooth: i32) -> Self {
        Self { time, speed, smooth}
    }
}