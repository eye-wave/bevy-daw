use super::node::{AudioNode, Connection, NodeId};
use hashbrown::HashMap;

pub struct EditorGraph {
    nodes: HashMap<NodeId, Box<dyn AudioNode>>,
    connections: Vec<(NodeId, NodeId)>,
}
pub struct RuntimeGraph {
    nodes: Vec<*mut dyn AudioNode>,
    sortec_connections: Vec<Connection>,
}

impl EditorGraph {
    pub fn construct(&self) -> RuntimeGraph {
        let mut nodes: Vec<*mut dyn AudioNode> = Vec::with_capacity(self.nodes.len());
        let mut id_to_index = HashMap::with_capacity(self.nodes.len());

        for (idx, (&id, node)) in self.nodes.iter().enumerate() {
            id_to_index.insert(id, idx);

            let ptr = &**node as *const dyn AudioNode as *mut dyn AudioNode;
            nodes.push(ptr);
        }

        let mut sorted_connections = Vec::with_capacity(self.connections.len());
        for &(source, target) in &self.connections {
            if id_to_index.contains_key(&source) && id_to_index.contains_key(&target) {
                sorted_connections.push(Connection::connect(source, target));
            }
        }

        RuntimeGraph {
            nodes,
            sortec_connections: sorted_connections,
        }
    }
}

impl RuntimeGraph {
    pub fn process<S>(&mut self, output: &mut [S])
    where
        S: cpal::Sample + cpal::FromSample<f32>,
    {
    }
}
