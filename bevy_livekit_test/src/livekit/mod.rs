use bevy::prelude::*;

mod resource;
pub use resource::RTCResource;

mod video;
mod video_renderer;

mod systems;
use systems::*;

pub struct LivekitPlugin;

impl Plugin for LivekitPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<RTCResource>()
        .add_systems(Update, wait_room_created)
        .add_systems(Update, on_event_received);
    }
}