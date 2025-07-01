use crate::{engine::BUFFER_SIZE, node::AudioNode};

#[derive(Debug)]
pub struct GroupNode {
    buffer: [f32; BUFFER_SIZE],
    nodes: Vec<Box<dyn AudioNode>>,
}

impl GroupNode {
    pub fn new() -> Self {
        Self {
            buffer: [0.0; BUFFER_SIZE],
            nodes: Vec::new(),
        }
    }

    pub fn add_node<A>(mut self, node: A) -> Self
    where
        A: AudioNode + 'static,
    {
        self.nodes.push(Box::new(node));
        self
    }
}

impl AudioNode for GroupNode {
    fn process(&mut self, sample_pos: u32, output: &mut [f32]) {
        self.buffer.fill(0.0);

        for node in &mut self.nodes {
            node.process(sample_pos, &mut self.buffer);
        }

        for (i, sample) in output.iter_mut().enumerate() {
            *sample += self.buffer[i];
        }
    }
}

impl Default for GroupNode {
    fn default() -> Self {
        Self::new()
    }
}
