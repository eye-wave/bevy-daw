use assert_no_alloc::*;
use bevy::ecs::resource::Resource;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::cell::RefCell;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub enum AudioCommand {
    Play(Box<[f32]>),
}

#[derive(Resource)]
pub struct AudioPlayer {
    pub sample_rate: f32,
    _stream: cpal::Stream,
    tx: Sender<AudioCommand>,
}

thread_local! {
    static CURRENT: RefCell<Option<(Box<[f32]>, usize)>> = const { RefCell::new(None) };
}

macro_rules! build_stream_match {
    ($device:expr, $config:expr, $rx:expr, $err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    &$config,
                    move |data: &mut [$ty], _| Self::process(data, &$rx),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    };
}

impl AudioPlayer {
    pub fn new() -> Self {
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
        let stream = build_stream_match!(
            device,
            config,
            rx,
            err_fn,
            {
                cpal::SampleFormat::F32 => f32,
                cpal::SampleFormat::I16 => i16,
                cpal::SampleFormat::U16 => u16,
                cpal::SampleFormat::U8 => u8,
            }
        )
        .expect("Failed to build output stream");

        stream.play().expect("Failed to start stream");

        Self {
            _stream: stream,
            tx,
            sample_rate,
        }
    }

    pub fn play(&self, samples: Box<[f32]>) {
        self.tx.send(AudioCommand::Play(samples)).unwrap();
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

    fn process<S>(output: &mut [S], rx: &Receiver<AudioCommand>)
    where
        S: cpal::Sample + cpal::FromSample<f32>,
    {
        assert_no_alloc(|| {
            CURRENT.with(|current| {
                let mut current = current.borrow_mut();

                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        AudioCommand::Play(samples) => {
                            *current = Some((samples, 0));
                        }
                    }
                }

                if let Some((ref samples, ref mut idx)) = *current {
                    for out in output.iter_mut() {
                        if *idx < samples.len() {
                            let sample = samples[*idx];
                            if sample.is_finite() {
                                *out = S::from_sample(samples[*idx].clamp(-1.0, 1.0));
                            } else {
                                *out = S::EQUILIBRIUM;
                            }

                            *idx += 1;
                        } else {
                            *out = S::EQUILIBRIUM;
                        }
                    }
                    if *idx >= samples.len() {
                        // remove later
                        assert_no_alloc::permit_alloc(|| {
                            *current = None;
                        });
                    }
                } else {
                    for out in output.iter_mut() {
                        *out = S::EQUILIBRIUM;
                    }
                }
            });
        });
    }
}
