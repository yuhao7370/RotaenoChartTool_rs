use serde::{Serialize, Serializer, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct TrailDistance {
    pub time: f32,
    pub degree: f32,
    pub delta: f32,
    pub prev_curv: f32,
    pub next_curv: f32,
    pub distance: f32,
}

impl TrailDistance {
    pub fn new(time: f32, degree: f32, delta: f32, prev_curv: f32, next_curv: f32, distance: f32) -> Self {
        Self { time, degree, delta, prev_curv, next_curv, distance}
    }
}