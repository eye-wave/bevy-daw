use crate::audio::AudioPlayer;
use bevy::prelude::*;

mod audio;

const TAU: f32 = 6.283_185_5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AudioPlayer::new())
        .add_systems(Startup, play_beep)
        .run();
}

fn play_beep(player: Res<AudioPlayer>) {
    let samples = generate_sine_wave(440.0, 2.0, player.sample_rate);

    player.play(samples);
}

fn generate_sine_wave(freq: f32, duration_secs: f32, sample_rate: f32) -> Box<[f32]> {
    let len = (duration_secs * sample_rate) as usize;
    (0..len)
        .map(|i| {
            // Play a kick sound because sin wave is boring
            let x = i as f32 * TAU * freq / sample_rate;
            let gain = 0.5;

            let sample = (20.0 * x.ln()).sin();

            sample * gain
        })
        .collect()
}
