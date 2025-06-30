use crate::EditorGraph;
use crate::graph::RuntimeGraph;
use assert_no_alloc::*;
use bevy::ecs::resource::Resource;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender, unbounded};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub const BUFFER_SIZE: usize = 2048;

struct SharedState {
    graph: Option<RuntimeGraph>,
    ring: AudioRingBuffer,
}

thread_local! {
    static CURRENT_GRAPH: RefCell<Option<RuntimeGraph>> = const { RefCell::new(None) };
    static AUDIO_RING: RefCell<Option<AudioRingBuffer>> = const { RefCell::new(None) };
}

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[derive(Debug)]
enum AudioCommand {
    LoadGraph(EditorGraph),
    ClearGraph,
}

pub(crate) struct AudioPlayer {
    _stream: cpal::Stream,
    rx: Receiver<AudioCommand>,
}

#[derive(Resource)]
pub struct AudioPlayerHandler {
    pub sample_rate: f32,
    tx: Sender<AudioCommand>,
}

pub struct AudioRingBuffer {
    producer: HeapProd<f32>,
    consumer: HeapCons<f32>,
}

macro_rules! build_stream_match {
    ($device:expr, $config:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    &$config,
                    move |data: &mut [$ty], _| Self::process(data),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    };
}

impl AudioPlayer {
    pub fn new() -> (Self, AudioPlayerHandler) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");

        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");
        let supported_config =
            Self::pick_config(&mut supported_configs).expect("No supported config");

        let sample_rate = supported_config.sample_rate().0 as f32;
        let config: cpal::StreamConfig = supported_config.into();

        let (tx, rx) = unbounded::<AudioCommand>();

        let err_fn = |err| eprintln!("Stream error: {err}");
        let _stream = build_stream_match!(
            device,
            config,
            err_fn,
            {
                cpal::SampleFormat::F32 => f32,
                cpal::SampleFormat::I16 => i16,
                cpal::SampleFormat::I32 => i32,
                cpal::SampleFormat::I8 => i8,
                cpal::SampleFormat::U16 => u16,
                cpal::SampleFormat::U32 => u32,
                cpal::SampleFormat::U8 => u8,
            }
        )
        .expect("Failed to build output stream");

        _stream.play().expect("Failed to start stream");

        AUDIO_RING.with(|ring| {
            let mut ring = ring.borrow_mut();

            *ring = Some(AudioRingBuffer::new());
        });

        (Self { _stream, rx }, AudioPlayerHandler { sample_rate, tx })
    }

    pub fn spawn_keep_alive(self) {
        std::thread::spawn(move || {
            let _self = self;
            loop {
                match _self.rx.recv_timeout(std::time::Duration::from_millis(10)) {
                    Ok(cmd) => _self.on_command(cmd),
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                    Err(e) => {
                        eprintln!("Channel recv error: {:?}", e);
                        break;
                    }
                }
            }
        });
    }

    fn pick_config(
        configs: &mut cpal::SupportedOutputConfigs,
    ) -> Option<cpal::SupportedStreamConfig> {
        // Best pick 44.1k and f32
        if let Some(config) = configs.find(|c| {
            c.min_sample_rate().0 <= 44100
                && c.max_sample_rate().0 >= 44100
                && c.sample_format() == cpal::SampleFormat::F32
        }) {
            return Some(config.with_sample_rate(cpal::SampleRate(44100)));
        }

        // at least 44.1k
        if let Some(config) =
            configs.find(|c| c.min_sample_rate().0 <= 44100 && c.max_sample_rate().0 >= 44100)
        {
            return Some(config.with_sample_rate(cpal::SampleRate(44100)));
        }

        configs
            .into_iter()
            .next()
            .map(|c| c.with_sample_rate(c.min_sample_rate()))
    }

    fn on_command(&self, cmd: AudioCommand) {
        println!("got command: {cmd:?}");

        match cmd {
            AudioCommand::LoadGraph(graph) => {
                let runtime_graph = graph.construct();

                CURRENT_GRAPH.with(|g| *g.borrow_mut() = Some(runtime_graph));
            }
            AudioCommand::ClearGraph => todo!(),
        }
    }

    fn process<S>(output: &mut [S])
    where
        S: cpal::Sample + cpal::FromSample<f32>,
    {
        let mut scratch = vec![0.0f32; output.len()];

        CURRENT_GRAPH.with(|current| {
            assert_no_alloc(|| {
                if let Some(graph) = &mut *current.borrow_mut() {
                    AUDIO_RING.with(|ring| {
                        let mut ring = ring.borrow_mut();
                        if let Some(ring) = ring.as_mut() {
                            while ring.available_samples() < output.len() {
                                let mut block = [0.0f32; BUFFER_SIZE];
                                graph.process(&mut block);
                                ring.write(&block);
                            }

                            ring.read(&mut scratch);
                            for (out, &s) in output.iter_mut().zip(scratch.iter()) {
                                *out = S::from_sample(s);
                            }
                        }
                    });
                } else {
                    for out in output.iter_mut() {
                        *out = S::EQUILIBRIUM;
                    }
                }
            });
        });
    }
}

impl AudioPlayerHandler {
    pub fn load_graph(&self, graph: EditorGraph) {
        self.tx.send(AudioCommand::LoadGraph(graph)).ok();
    }

    pub fn clear_graph(&self) {
        self.tx.send(AudioCommand::ClearGraph).ok();
    }
}

impl AudioRingBuffer {
    pub fn new() -> Self {
        let rb = HeapRb::<f32>::new(BUFFER_SIZE);
        let (producer, consumer) = rb.split();

        Self { producer, consumer }
    }

    pub fn write(&mut self, input: &[f32]) -> usize {
        self.producer.push_slice(input)
    }

    pub fn read(&mut self, output: &mut [f32]) -> usize {
        self.consumer.pop_slice(output)
    }

    pub fn available_samples(&self) -> usize {
        self.consumer.occupied_len()
    }
}
