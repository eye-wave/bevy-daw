use crate::graph::RuntimeGraph;
use assert_no_alloc::*;
use bevy::{app::Plugin, ecs::resource::Resource};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::cell::RefCell;

thread_local! {
    static CURRENT_GRAPH: RefCell<Option<RuntimeGraph>> = const { RefCell::new(None) };
    static PENDING_GRAPH: RefCell<Option<RuntimeGraph>> = const { RefCell::new(None) };
}

mod graph;
mod node;

pub use graph::EditorGraph;

pub struct DawPlugin;

impl Plugin for DawPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let (player, handler) = AudioPlayer::new();
        player.spawn_keep_alive();

        app.insert_resource(handler);
    }
}

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub enum AudioCommand {
    LoadGraph(EditorGraph),
    ClearGraph,
}

struct AudioPlayer {
    _stream: cpal::Stream,
    rx: Receiver<AudioCommand>,
}

#[derive(Resource)]
pub struct AudioPlayerHandler {
    pub sample_rate: f32,
    tx: Sender<AudioCommand>,
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

        (Self { _stream, rx }, AudioPlayerHandler { sample_rate, tx })
    }

    pub fn spawn_keep_alive(self) {
        std::thread::spawn(move || {
            let _self = self;

            loop {
                if let Ok(cmd) = _self.rx.try_recv() {
                    _self.on_command(cmd);
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
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
        match cmd {
            AudioCommand::LoadGraph(graph) => {
                let runtime_graph = graph.construct();

                PENDING_GRAPH.with(|p| *p.borrow_mut() = Some(runtime_graph));
            }
            AudioCommand::ClearGraph => todo!(),
        }
    }

    fn process<S>(output: &mut [S])
    where
        S: cpal::Sample + cpal::FromSample<f32>,
    {
        PENDING_GRAPH.with(|pending| {
            if let Some(graph) = pending.borrow_mut().take() {
                CURRENT_GRAPH.with(|current| *current.borrow_mut() = Some(graph));
            }
        });

        CURRENT_GRAPH.with(|current| {
            assert_no_alloc(|| {
                if let Some(graph) = &mut *current.borrow_mut() {
                    graph.process(output);
                } else {
                    for out in output.iter_mut() {
                        *out = S::EQUILIBRIUM;
                    }
                }
            });
        });
    }
}
