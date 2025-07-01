use bevy::prelude::*;
use bevy_daw::{
    AudioEngine, DawPlugin,
    nodes::{GroupNode, ToneGeneratorNode},
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
        let group = GroupNode::new().add_node(ToneGeneratorNode::new(440.0, 0.3));

        graph.nodes.push(Box::new(group));
    });
}
