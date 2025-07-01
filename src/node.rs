use std::fmt::Debug;

mod delay;
mod distortion;
mod gain;
mod group;
mod tone;

pub trait AudioNode: Send + Debug {
    fn process(&mut self, sample_pos: u32, output: &mut [f32]);
}

pub mod nodes {
    pub use super::delay::*;
    pub use super::distortion::*;
    pub use super::gain::*;
    pub use super::group::*;
    pub use super::tone::*;
}
