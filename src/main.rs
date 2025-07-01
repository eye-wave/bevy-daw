use bevy::prelude::*;
use bevy_daw::{
    AudioEngine, DawPlugin, MidiNote,
    nodes::{DistortionNode, DistortionType, GroupNode, ToneGeneratorNode},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .add_systems(Startup, play_something)
        .run();
}

fn play_something(player: Res<AudioEngine>) {
    player.edit_graph(|graph| {
        let group = GroupNode::new()
            .add_node(ToneGeneratorNode::new(MidiNote::new(60), 0.3))
            .add_node(DistortionNode::new(10.0, 0.4, DistortionType::SoftClip))
            .add_node(ToneGeneratorNode::new(MidiNote::new(65), 0.2))
            .add_node(ToneGeneratorNode::new(MidiNote::new(68), 0.2));

        graph.nodes.push(Box::new(group));
    });
}
