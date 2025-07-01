use super::traits::AudioNode;
use assert_no_alloc::*;
use bevy::ecs::resource::Resource;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::StaticRb;
use ringbuf::traits::{Consumer, Producer, Split};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub const SAMPLE_RATE: u32 = 44_100;
pub const BUFFER_SIZE: usize = 4096;

#[derive(Default, Debug)]
pub struct EngineState {
    pub nodes: Vec<Box<dyn AudioNode>>,
}

impl EngineState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(EngineState { nodes: Vec::new() }))
    }
}

impl AudioNode for EngineState {
    fn process(&mut self, sample_pos: u32, buf: &mut [f32]) {
        buf.fill(0.0);

        for node in &mut self.nodes {
            node.process(sample_pos, buf);
        }
    }
}

#[derive(Resource)]
pub struct AudioEngine {
    state: Arc<Mutex<EngineState>>,
}

macro_rules! build_stream_match {
    ($device:expr, $config:expr, $consumer:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    move |data: &mut [$ty], _| {
                        assert_no_alloc(|| {
                            for sample in data.iter_mut() {
                                *sample = resample($consumer.try_pop().unwrap_or(0.0));
                            }
                        })
                    },
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    };
}

impl AudioEngine {
    pub fn new() -> Self {
        let state = EngineState::new();
        let state_for_thread = state.clone();

        let rb = StaticRb::<f32, BUFFER_SIZE>::from([0.0; BUFFER_SIZE]);
        let (mut producer, mut consumer) = rb.split();

        // Setup audio output
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");

        let config = pick_config(&mut supported_configs).unwrap().into();

        // Consumer
        let stream = build_stream_match!(
            device,
            &config,
            consumer,
            |err| eprintln!("{err}"),
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
        // Producer
        let prod = thread::spawn(move || {
            assert_no_alloc(|| {
                let mut sample_pos = 0;
                let mut buf = [0.0f32; BUFFER_SIZE];

                loop {
                    {
                        let mut state = state_for_thread.lock().unwrap();

                        state.process(sample_pos, &mut buf);
                        sample_pos += BUFFER_SIZE as u32;
                    }

                    let mut written = 0;
                    while written < buf.len() {
                        if producer.try_push(buf[written]).is_ok() {
                            written += 1;
                        } else {
                            thread::yield_now();
                        }
                    }
                }
            });
        });

        stream.play().unwrap();

        thread::spawn(|| {
            let _prod = prod;
            let _stream = stream;

            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });

        Self { state }
    }

    pub fn edit_graph<F>(&self, f: F)
    where
        F: FnOnce(&mut EngineState),
    {
        let mut state = self.state.lock().unwrap();
        f(&mut state);
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn resample<S>(sample: f32) -> S
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    S::from_sample(sample)
}

fn pick_config(configs: &mut cpal::SupportedOutputConfigs) -> Option<cpal::SupportedStreamConfig> {
    // Best pick 44.1k and f32
    if let Some(config) = configs.find(|c| {
        c.min_sample_rate().0 <= SAMPLE_RATE
            && c.max_sample_rate().0 >= SAMPLE_RATE
            && c.sample_format() == cpal::SampleFormat::F32
    }) {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    // at least 44.1k
    if let Some(config) = configs
        .find(|c| c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
    {
        return Some(config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)));
    }

    None
}
