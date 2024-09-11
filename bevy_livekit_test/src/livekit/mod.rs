use bevy::prelude::*;

mod service;
mod video;

mod resource;
pub use resource::*;

mod systems;
pub use systems::*;

pub struct LKPlugin;

impl Plugin for LKPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<LKResource>()
        .add_systems(Update, on_room_event_received);
    }
}