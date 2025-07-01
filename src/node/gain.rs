use crate::node::AudioNode;

#[derive(Debug)]
pub struct GainNode {
    gain: f32,
}

impl GainNode {
    pub fn new(gain: f32) -> Self {
        Self { gain }
    }
}

impl AudioNode for GainNode {
    fn process(&mut self, _sample_pos: u32, output: &mut [f32]) {
        for sample in output {
            *sample *= self.gain;
        }
    }
}

impl Default for GainNode {
    fn default() -> Self {
        Self::new(1.0)
    }
}
