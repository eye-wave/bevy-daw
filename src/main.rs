use bevy::prelude::*;
use bevy_daw::DawPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .run();
}
