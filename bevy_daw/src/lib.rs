use bevy::app::Plugin;

mod engine;
mod node;
pub struct DawPlugin;

pub use engine::AudioEngine;
pub use node::nodes;

impl Plugin for DawPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(AudioEngine::default());
    }
}

pub mod traits {
    pub use super::node::AudioNode;
}
