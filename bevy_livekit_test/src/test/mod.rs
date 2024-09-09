use bevy::prelude::*;

mod systems;
use systems::*;

mod resource;
pub use resource::*;

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<MenuResource>()
        .add_systems(Update, test_menu);
    }
}