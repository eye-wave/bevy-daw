#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u32);

pub const OUTPUT_NODE_ID: NodeId = NodeId(0);

pub struct Connection(u64);

impl Connection {
    pub fn connect(source: NodeId, target: NodeId) -> Self {
        let source_u = source.0 as u64;
        let target_u = target.0 as u64;

        Connection((source_u << 32) | target_u)
    }

    pub fn source(&self) -> NodeId {
        NodeId((self.0 >> 32) as u32)
    }

    pub fn target(&self) -> NodeId {
        NodeId((self.0 & 0xFFFF_FFFF) as u32)
    }
}

pub trait AudioNode: Send {
    fn process(&mut self, output: &mut [f32]);
}

pub struct NodeIdGenerator {
    next_id: u32,
}

impl NodeIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn generate(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }
}
