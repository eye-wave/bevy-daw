use super::SAMPLE_RATE;
use crate::AudioController;
use crate::engine::{AUDIO_STATE, MAX_BUFFER_SIZE};
use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hashbrown::HashMap;
use std::thread;
use std::time::Duration;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn audio_loop<S>(data: &mut [S])
where
    S: cpal::Sample + cpal::FromSample<f32>,
{
    assert_no_alloc(|| {
        AUDIO_STATE.with(|state| {
            let buf = &mut [0.0f32; MAX_BUFFER_SIZE];
            let mut processed = false;

            if let Ok(mut state) = state.try_borrow_mut() {
                state.process(&mut buf[..data.len()]);
                processed = true;
            }

            for i in 0..data.len() {
                let sample = if processed { buf[i] } else { 0.0 };
                data[i] = resample(sample);
            }
        });
    });
}

macro_rules! build_stream_match {
    ($device:expr, $config:expr, $state:expr,$err_fn:expr, { $( $fmt:path => $ty:ty ),* $(,)? }) => {
        match $device.default_output_config().unwrap().sample_format() {
            $(
                $fmt => $device.build_output_stream(
                    $config,
                    |data: &mut [$ty], _| audio_loop(data),
                    $err_fn,
                    None,
                ),
            )*
            other => panic!("Unsupported sample format {:?}", other),
        }
    };
}

impl AudioController {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device");
        let mut supported_configs = device
            .supported_output_configs()
            .expect("Error getting configs");

        let config = pick_config(&mut supported_configs);
        if config.is_none() {
            return Self {
                ..Default::default()
            };
        }

        let stream = build_stream_match!(
            device,
            &config.unwrap().into(),
            state_for_thread,
            |err| eprintln!("{err}"),
            {
                cpal::SampleFormat::F32 => f32,
                cpal::SampleFormat::I16 => i16,
                cpal::SampleFormat::I24 => cpal::I24,
                cpal::SampleFormat::I32 => i32,
                cpal::SampleFormat::I8 => i8,
                cpal::SampleFormat::U16 => u16,
                cpal::SampleFormat::U32 => u32,
                cpal::SampleFormat::U8 => u8,
            }
        )
        .expect("Failed to build output stream");

        stream.play().unwrap();

        thread::spawn(|| {
            let _stream = stream;

            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });

        Self {
            ..Default::default()
        }
    }
}

impl Default for AudioController {
    fn default() -> Self {
        Self {
            next_id: 0,
            nodes: HashMap::new(),
        }
    }
}

pub(super) fn resample<S>(sample: f32) -> S
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
