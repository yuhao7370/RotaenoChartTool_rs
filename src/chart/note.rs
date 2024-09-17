// note.rs
use serde::{Serialize, Serializer, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tap {
    pub time: f32,
    pub degree: f32,
}

impl Tap {
    pub fn new(time: f32, degree: f32) -> Self {
        Self { time, degree }
    }

}

#[derive(Serialize, Deserialize, Clone)]
pub struct Flick {
    pub time: f32,
    pub degree: f32,
}

impl Flick {
    pub fn new(time: f32, degree: f32) -> Self {
        Self { time, degree }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Slide {
    pub time: f32,
    pub degree: f32,
    pub slidetype: i32,
    pub end_degree: f32,
    pub snap: i32,
    pub amount: i32,
    pub prev_curv: f32,
    pub next_curv: f32,
}

impl Slide {
    pub fn new(time: f32, degree: f32, slidetype: i32, end_degree: f32, snap: i32, amount: i32, prev_curv: f32, next_curv: f32) -> Self {
        Self { time, degree, slidetype, end_degree, snap, amount, prev_curv, next_curv }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rotate {
    pub time: f32,
    pub degree: f32,
    pub delta: f32,
    pub prev_curv: f32,
    pub next_curv: f32,
}

impl Rotate {
    pub fn new(time: f32, degree: f32, delta: f32, prev_curv: f32, next_curv: f32) -> Self {
        Self { time, degree, delta, prev_curv, next_curv }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Catch {
    pub time: f32,
    pub degree: f32,
}

impl Catch {
    pub fn new(time: f32, degree: f32) -> Self {
        Self { time, degree }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trail {
    pub time: f32,
    pub degree: f32,
    pub delta: f32,
    pub prev_curv: f32,
    pub next_curv: f32,
}

impl Trail {
    pub fn new(time: f32, degree: f32, delta: f32, prev_curv: f32, next_curv: f32) -> Self {
        Self { time, degree, delta, prev_curv, next_curv}
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bomb {
    pub time: f32,
    pub degree: f32,
}

impl Bomb {
    pub fn new(time: f32, degree: f32) -> Self {
        Self { time, degree }
    }
}

#[derive(Deserialize, Clone)]
pub enum Note {
    Tap(Tap),
    Flick(Flick),
    Slide(Slide),
    Rotate(Rotate),
    Catch(Catch),
    Trail(Trail),
    Bomb(Bomb),
}

impl Serialize for Note {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Note::Trail(_) => serializer.serialize_none(),
            Note::Tap(tap) => tap.serialize(serializer),
            Note::Flick(flick) => flick.serialize(serializer),
            Note::Slide(slide) => slide.serialize(serializer),
            Note::Rotate(rotate) => rotate.serialize(serializer),
            Note::Catch(catch) => catch.serialize(serializer),
            Note::Bomb(bomb) => bomb.serialize(serializer),
        }
    }
}
impl Note {
    pub fn get_time(&self) -> f32 {
        match self {
            Note::Tap(tap) => tap.time,
            Note::Flick(flick) => flick.time,
            Note::Slide(slide) => slide.time,
            Note::Rotate(rotate) => rotate.time,
            Note::Catch(catch) => catch.time,
            Note::Trail(trail) => trail.time,
            Note::Bomb(bomb) => bomb.time,
        }
    }
}