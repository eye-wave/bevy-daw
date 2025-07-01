use bevy::prelude::*;
use bevy_daw::{AudioEngine, DawPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .add_systems(Startup, beep)
        .run();
}

fn beep(player: Res<AudioEngine>) {
    player.set_frequency(660.0);
}
