use bevy::prelude::*;

mod test;
use bevy_egui::EguiPlugin;
use test::TestPlugin;

mod livekit;
use livekit::LivekitPlugin;

fn main() {

    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin)
    .add_plugins(LivekitPlugin)
    .add_plugins(TestPlugin)
    .run();
}
