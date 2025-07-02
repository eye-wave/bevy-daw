use super::traits::AudioNode;
use crate::node::NodeId;
use bevy::ecs::resource::Resource;
use hashbrown::HashMap;
use heapless::spsc::Queue;
use spin::Mutex;
use std::cell::RefCell;

mod device;

pub const SAMPLE_RATE: u32 = 44_100;
pub const MAX_BUFFER_SIZE: usize = 8192;

#[derive(Debug)]
pub struct NodePtr<T: ?Sized>(*mut T);

unsafe impl<T: ?Sized + Send> Send for NodePtr<T> {}
unsafe impl<T: ?Sized + Sync> Sync for NodePtr<T> {}

impl<T: ?Sized> NodePtr<T> {
    pub unsafe fn new(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() { None } else { Some(Self(ptr)) }
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0 }
    }

    pub unsafe fn into_box(self) -> Box<T> {
        unsafe { Box::from_raw(self.0) }
    }
}

impl<T: ?Sized> Clone for NodePtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[derive(Debug)]
pub(super) enum AudioCommand {
    AddNode(NodePtr<dyn AudioNode>),
    RemoveNode(NodePtr<dyn AudioNode>),
}

thread_local! {
    static AUDIO_STATE: RefCell<AudioEngine> = RefCell::new(AudioEngine::empty());
}

static AUDIO_QUEUE: Mutex<Queue<AudioCommand, 64>> = Mutex::new(Queue::new());

#[derive(Default, Debug)]
pub struct AudioEngine {
    nodes: heapless::Vec<NodePtr<dyn AudioNode>, 256>,
    sample_pos: u32,
}

impl AudioEngine {
    pub fn empty() -> Self {
        Self {
            sample_pos: 0,
            nodes: heapless::Vec::new(),
        }
    }

    fn on_command(&mut self, cmd: AudioCommand) {
        match cmd {
            AudioCommand::AddNode(ptr) => {
                self.nodes.push(ptr).ok();
            }
            AudioCommand::RemoveNode(ptr) => {
                self.nodes.retain(|node| !std::ptr::addr_eq(node.0, ptr.0));
            }
        };
    }

    fn process(&mut self, buf: &mut [f32]) {
        buf.fill(0.0);

        let mut queue = AUDIO_QUEUE.lock();
        let (_, mut consumer) = queue.split();

        while let Some(cmd) = consumer.dequeue() {
            self.on_command(cmd);
        }

        for node_ptr in &mut self.nodes {
            unsafe {
                node_ptr.as_mut().process(self.sample_pos, buf);
            }
        }

        self.sample_pos = self.sample_pos.wrapping_add(buf.len() as u32);
    }
}

#[derive(Debug, Resource)]
pub struct AudioController {
    nodes: HashMap<NodeId, NodePtr<dyn AudioNode>>,
    next_id: u32,
}

impl AudioController {
    pub fn add_node(&mut self, node: Box<dyn AudioNode>) -> Option<NodeId> {
        unsafe {
            let static_node: &'static mut dyn AudioNode = Box::leak(node);

            if let Some(ptr) = NodePtr::new(static_node as *mut dyn AudioNode) {
                let id = NodeId(self.next_id + 1);
                let ptr_copy = ptr.clone();

                self.nodes.insert(id, ptr);
                self.send_command(AudioCommand::AddNode(ptr_copy));

                self.next_id += 1;

                return Some(id);
            } else {
                let boxed = Box::from_raw(static_node);
                drop(boxed);
            }
        }

        None
    }

    pub fn remove_node(&mut self, id: NodeId) {
        if let Some(ptr) = self.nodes.get(&id) {
            let boxed: Box<dyn AudioNode> = unsafe { ptr.clone().into_box() };
            self.send_command(AudioCommand::RemoveNode(ptr.clone()));
            drop(boxed)
        }
    }

    pub(super) fn send_command(&self, cmd: AudioCommand) {
        let mut queue = AUDIO_QUEUE.lock();
        let (mut producer, _) = queue.split();
        producer.enqueue(cmd).ok();
    }
}
