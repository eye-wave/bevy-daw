use bevy::app::Plugin;

mod engine;
mod node;
mod utils;

pub struct DawPlugin;

pub use engine::AudioEngine;
pub use node::nodes;

pub use utils::MidiNote;

impl Plugin for DawPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(AudioEngine::default());
    }
}

pub mod traits {
    pub use super::node::AudioNode;
    pub use super::utils::Note;
}
