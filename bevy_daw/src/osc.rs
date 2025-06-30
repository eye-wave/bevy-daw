use crate::{NodeId, node::AudioNode};
use std::f32::consts::TAU;

#[derive(Debug)]
pub struct TestTone {
    id: NodeId,
    freq: f32,
    phase: f32,
}

impl TestTone {
    pub fn new(freq: f32, id: u32) -> Self {
        Self {
            freq,
            id: NodeId(id),
            phase: 0.0,
        }
    }
}

impl AudioNode for TestTone {
    fn id(&self) -> NodeId {
        self.id
    }

    fn process(&mut self, output: &mut [f32], sample_rate: f32) {
        let phase_inc = self.freq / sample_rate;

        for sample in output.iter_mut() {
            *sample = (self.phase * TAU).sin();
            self.phase += phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}
