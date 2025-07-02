use bevy::prelude::*;
use bevy_daw::{
    AudioController, DawPlugin, NodeId,
    nodes::{DistortionNode, DistortionType, GroupNode, ToneGeneratorNode},
};
use std::time::Duration;

#[derive(Component)]
struct TimedNode {
    node_id: NodeId,
    timer: Timer,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DawPlugin)
        .add_systems(Startup, play_something)
        .add_systems(Update, timed_node_cleanup)
        .run();
}

fn play_something(mut player: ResMut<AudioController>, mut commands: Commands) {
    let group = GroupNode::new()
        .add_node(ToneGeneratorNode::new(440.0, 1.0))
        .add_node(DistortionNode::new(10.0, 0.2, DistortionType::SoftClip));

    if let Some(id) = player.add_node(Box::new(group)) {
        commands.spawn(TimedNode {
            node_id: id,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Once),
        });
    }
}

fn timed_node_cleanup(
    time: Res<Time>,
    mut player: ResMut<AudioController>,
    mut query: Query<(Entity, &mut TimedNode)>,
    mut commands: Commands,
) {
    for (entity, mut timed_node) in query.iter_mut() {
        timed_node.timer.tick(time.delta());
        if timed_node.timer.finished() {
            player.remove_node(timed_node.node_id);
            commands.entity(entity).despawn();
        }
    }
}
