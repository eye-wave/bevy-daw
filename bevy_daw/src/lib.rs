use bevy::app::Plugin;

mod engine;
pub mod nodes;

pub struct DawPlugin;
pub use engine::AudioEngine;

impl Plugin for DawPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(AudioEngine::default());
    }
}
