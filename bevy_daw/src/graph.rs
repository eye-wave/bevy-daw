use super::node::{AudioNode, Connection, NodeId};
use hashbrown::HashMap;

#[derive(Debug, Default)]
pub struct EditorGraph {
    nodes: HashMap<NodeId, Box<dyn AudioNode>>,
    connections: Vec<Connection>,
}

#[derive(Debug)]
pub struct RuntimeGraph {
    sample_pos: u32,
    nodes: Vec<*mut dyn AudioNode>,
    sortec_connections: Vec<Connection>,
}

impl EditorGraph {
    pub fn add(&mut self, source: Box<dyn AudioNode>) {
        if !self.nodes.contains_key(&source.id()) {
            self.nodes.insert(source.id(), source);
        }
    }

    pub fn connect(&mut self, source: NodeId, target: NodeId) {
        let connection = Connection::connect(source, target);
        if !self.connections.contains(&connection) {
            self.connections.push(connection);
        }
    }

    pub fn construct(&self) -> RuntimeGraph {
        let mut nodes: Vec<*mut dyn AudioNode> = Vec::with_capacity(self.nodes.len());
        let mut id_to_index = HashMap::with_capacity(self.nodes.len());

        for (idx, (&id, node)) in self.nodes.iter().enumerate() {
            id_to_index.insert(id, idx);

            let ptr = &**node as *const dyn AudioNode as *mut dyn AudioNode;
            nodes.push(ptr);
        }

        let mut sorted_connections = Vec::with_capacity(self.connections.len());
        for conn in self.connections.iter() {
            if id_to_index.contains_key(&conn.source()) && id_to_index.contains_key(&conn.target())
            {
                sorted_connections.push(Connection::connect(conn.source(), conn.target()));
            }
        }

        RuntimeGraph {
            sample_pos: 0,
            nodes,
            sortec_connections: sorted_connections,
        }
    }
}

impl RuntimeGraph {
    pub fn process(&mut self, output: &mut [f32]) {
        let block_size = output.len() as u32;

        println!("{:?}", self.nodes);

        self.sample_pos += block_size;
    }
}
