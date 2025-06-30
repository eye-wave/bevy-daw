use bevy::prelude::*;
use bevy_daw::{AudioNode, AudioPlayerHandler, DawPlugin, EditorGraph, OUTPUT_NODE, TestTone};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .add_systems(Startup, beep)
        .run();
}

fn beep(player: Res<AudioPlayerHandler>) {
    let mut graph = EditorGraph::default();
    let test_tone = TestTone::new(440.0, 10);

    graph.connect(test_tone.id(), OUTPUT_NODE);
    graph.add(Box::new(test_tone));

    player.load_graph(graph);
}
