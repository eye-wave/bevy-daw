use crate::{engine::SAMPLE_RATE, node::AudioNode};
use std::f32::consts::TAU;

#[derive(Debug)]
pub struct ToneGeneratorNode {
    volume: f32,
    phase: f32,
    phase_inc: f32,
}

impl ToneGeneratorNode {
    pub fn new<N: Into<f32>>(freq: N, volume: f32) -> Self {
        let phase_inc = (freq.into() / SAMPLE_RATE as f32) * TAU;
        Self {
            volume,
            phase: 0.0,
            phase_inc,
        }
    }
}

impl AudioNode for ToneGeneratorNode {
    fn process(&mut self, _sample_pos: u32, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample += self.phase.sin() * self.volume;
            self.phase += self.phase_inc;

            if self.phase > TAU {
                self.phase -= TAU;
            }
        }
    }
}
