use crate::device::AudioPlayer;
use bevy::app::Plugin;

mod device;
mod graph;
mod node;
mod osc;

pub use device::AudioPlayerHandler;
pub use graph::EditorGraph;
pub use node::{AudioNode, Connection, NodeId, NodeIdGenerator, OUTPUT_NODE_ID as OUTPUT_NODE};
pub use osc::TestTone;

pub struct DawPlugin;

impl Plugin for DawPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let (player, handler) = AudioPlayer::new();
        player.spawn_keep_alive();

        app.insert_resource(handler);
    }
}
