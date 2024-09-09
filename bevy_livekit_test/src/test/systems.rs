use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::livekit::RTCResource;

use super::MenuResource;

pub fn test_menu(mut contexts: EguiContexts, mut menu_resource: ResMut<MenuResource>, mut rtc_resource: ResMut<RTCResource>){
    let ctx: &mut egui::Context = contexts.ctx_mut();

    egui::Window::new("menu")
    .title_bar(false)
    .show(ctx, |ui| {

        ui.text_edit_singleline(&mut menu_resource.username);

        if rtc_resource.is_multiplayer() {
            if ui.button("Disconnect").clicked() {
                rtc_resource.leave_room(&menu_resource.username)
            }

            if ui.button("Publish Video").clicked() {
                rtc_resource.new_video_track("testing track");
            }
        } else {
            if ui.button("Connect").clicked() {
                rtc_resource.new_room(&menu_resource.username);
            }
        }
    });
}