use crate::engine::MAX_BUFFER_SIZE;
use crate::node::AudioNode;

#[derive(Debug)]
pub struct DelayNode {
    delay_samples: usize,
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayNode {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            delay_samples,
            buffer: vec![0.0; delay_samples + MAX_BUFFER_SIZE],
            write_pos: 0,
        }
    }
}

impl AudioNode for DelayNode {
    fn process(&mut self, _sample_pos: u32, output: &mut [f32]) {
        for sample in output.iter_mut() {
            self.buffer[self.write_pos] = *sample;

            let read_pos =
                (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();

            *sample = self.buffer[read_pos];

            self.write_pos = (self.write_pos + 1) % self.buffer.len();
        }
    }
}
