//render/mod.rs
pub mod texturemanager;
pub use texturemanager::TextureManager;

pub mod noterenderer;
// pub use noterenderer::{dwaw_arc, distance_to_radius};

pub mod trailrenderer;
pub use trailrenderer::{draw_trail, distance_to_radius, dwaw_arc};