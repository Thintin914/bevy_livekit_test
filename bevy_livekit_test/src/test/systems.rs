use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::livekit::LKResource;

use super::MenuResource;

pub fn test_menu(mut contexts: EguiContexts, mut menu_resource: ResMut<MenuResource>, mut lk_resource: ResMut<LKResource>){
    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("menu")
    .title_bar(false)
    .show(ctx, |ui| {

        ui.text_edit_singleline(&mut menu_resource.username);

        if lk_resource.is_multiplayer() {
            if ui.button("Disconnect").clicked() {
                lk_resource.leave("test", &menu_resource.username);
            }

            if ui.button("Publish Video").clicked() {
                lk_resource.publish_video_track("testing track");
            }
        } else {
            if ui.button("Connect").clicked() {
                lk_resource.create_room("test", &menu_resource.username);
            }
        }
    });
}