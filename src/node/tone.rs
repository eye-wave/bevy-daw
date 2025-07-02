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

#[cfg(test)]
mod test {
    use super::{AudioNode, ToneGeneratorNode};
    use crate::engine::SAMPLE_RATE;
    use crate::node::test_utils::test::*;

    #[test]
    fn plot_tone_generator() {
        let freq = SAMPLE_RATE as f32 / 2048.0;
        let mut tone1 = ToneGeneratorNode::new(freq * 2.0, 0.5);
        let mut tone2 = ToneGeneratorNode::new(freq * 3.0, 0.5);

        let mut buffer = [0.0; 2048];

        tone1.process(0, &mut buffer);
        tone2.process(0, &mut buffer);

        node_test_suite(&buffer, 1024, "tone-generator");
    }
}
