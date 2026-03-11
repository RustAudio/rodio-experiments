mod chirp;
mod function;
mod noise;
mod silence;

pub mod const_source {
    pub use super::chirp::const_source::Chirp;
    pub use super::function::const_source as function;
    pub use super::noise::const_source as noise;
    pub use super::silence::const_source as silence;
}

pub mod fixed_source {
    pub use super::chirp::fixed_source::Chirp;
    pub use super::function::fixed_source as function;
    pub use super::noise::fixed_source as noise;
    pub use super::silence::fixed_source as silence;
}
