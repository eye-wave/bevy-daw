use super::traits::AudioNode;
use bevy::ecs::resource::Resource;
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
}

thread_local! {
    static AUDIO_STATE: RefCell<EngineState> = RefCell::new(EngineState::empty());
}

static AUDIO_QUEUE: Mutex<Queue<NodePtr<dyn AudioNode>, 64>> = Mutex::new(Queue::new());

#[derive(Default, Debug)]
pub struct EngineState {
    nodes: heapless::Vec<NodePtr<dyn AudioNode>, 256>,
    sample_pos: u32,
}

impl EngineState {
    pub fn empty() -> Self {
        Self {
            sample_pos: 0,
            nodes: heapless::Vec::new(),
        }
    }

    fn process(&mut self, buf: &mut [f32]) {
        buf.fill(0.0);

        let mut queue = AUDIO_QUEUE.lock();
        let (_, mut consumer) = queue.split();

        while let Some(ptr) = consumer.dequeue() {
            let _ = self.nodes.push(ptr);
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
pub struct AudioEngine {}

impl AudioEngine {
    pub fn add_node(&mut self, node: Box<dyn AudioNode>) {
        unsafe {
            let static_node: &'static mut dyn AudioNode = Box::leak(node);
            if let Some(ptr) = NodePtr::new(static_node as *mut dyn AudioNode) {
                self.send_node_ptr(ptr);
            }
        }
    }

    pub(super) fn send_node_ptr(&self, ptr: NodePtr<dyn AudioNode>) {
        let mut queue = AUDIO_QUEUE.lock();
        let (mut producer, _) = queue.split();
        let _ = producer.enqueue(ptr);
    }
}
