use std::env::set_var;
use dotenvy_macro::dotenv;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod test;
use test::TestPlugin;

mod livekit;
use livekit::LKPlugin;

fn main() {

    set_var("AWS_ACCESS_KEY_ID", dotenv!("AWS_ACCESS_KEY_ID"));
    set_var("AWS_SECRET_ACCESS_KEY", dotenv!("AWS_SECRET_ACCESS_KEY"));
    set_var("AWS_REGION", dotenv!("AWS_REGION"));

    set_var("LIVEKIT_URL", dotenv!("LIVEKIT_URL"));
    set_var("LIVEKIT_API_KEY", dotenv!("LIVEKIT_API_KEY"));
    set_var("LIVEKIT_API_SECRET", dotenv!("LIVEKIT_API_SECRET"));

    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin)
    .add_plugins(TestPlugin)
    .add_plugins(LKPlugin)
    .run();
}
