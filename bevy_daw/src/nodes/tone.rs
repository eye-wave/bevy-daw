use crate::{engine::SAMPLE_RATE, nodes::AudioNode};
use std::f32::consts::TAU;

pub struct ToneGenerator {
    freq: f32,
    volume: f32,
}

impl ToneGenerator {
    pub fn new(freq: f32, volume: f32) -> Self {
        Self { freq, volume }
    }

    fn phase(&self, step: usize) -> f32 {
        (step as f32 * self.freq / SAMPLE_RATE as f32) * TAU
    }
}

impl AudioNode for ToneGenerator {
    fn process(&mut self, sample_pos: u32, output: &mut [f32]) {
        for (i, sample) in output.iter_mut().enumerate() {
            *sample += self.phase(sample_pos as usize + i).sin() * self.volume;
        }
    }
}
