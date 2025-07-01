use crate::{engine::SAMPLE_RATE, node::AudioNode};
use std::f32::consts::TAU;

#[derive(Debug)]
pub struct ToneGenerator {
    volume: f32,
    phase: f32,
    phase_inc: f32,
}

impl ToneGenerator {
    pub fn new(freq: f32, volume: f32) -> Self {
        let phase_inc = (freq / SAMPLE_RATE as f32) * TAU;
        Self {
            volume,
            phase: 0.0,
            phase_inc,
        }
    }
}

impl AudioNode for ToneGenerator {
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
