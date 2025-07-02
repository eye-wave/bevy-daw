use bevy::prelude::*;
use bevy_daw::{
    AudioEngine, DawPlugin,
    nodes::{DistortionNode, DistortionType, GroupNode, ToneGeneratorNode},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .add_systems(Startup, play_something)
        .run();
}

fn play_something(mut player: ResMut<AudioEngine>) {
    let group = GroupNode::new()
        .add_node(ToneGeneratorNode::new(440.0, 1.0))
        .add_node(DistortionNode::new(10.0, 0.2, DistortionType::SoftClip));

    player.add_node(Box::new(group));
}
