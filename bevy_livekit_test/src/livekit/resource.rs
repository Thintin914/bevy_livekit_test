use std::sync::Arc;

use bevy::prelude::*;
use image::RgbaImage;
use parking_lot::Mutex;

use super::service::{ConnectionState, LKService, RoomAction};

#[derive(Resource)]
pub struct LKResource {
    pub service: Arc<Mutex<Option<LKService>>>
}

impl Default for LKResource {
    fn default() -> LKResource {

        let lk_service = Arc::new(Mutex::new(Some(LKService::new())));

        LKResource {
            service: lk_service
        }
    }
}

impl LKResource {
    pub fn create_room(&mut self, room_id: &str, user_uuid: &str){
        if self.is_multiplayer() {
            return;
        }
        if let Some(lk_service) = self.service.lock().as_mut(){
            let _ = lk_service.input_sender.send(RoomAction::ConnectRoom { room_id: room_id.to_string(), user_uuid: user_uuid.to_string() });
        }
    }

    pub fn leave(&mut self, room_id: &str, user_uuid: &str){
        if !self.is_multiplayer() {
            return;
        }

        if let Some(lk_service) = self.service.lock().as_mut(){
            let _ = lk_service.input_sender.send(RoomAction::LeaveRoom { room_id: room_id.to_string(), user_uuid: user_uuid.to_string() });
        }
    }

    pub fn publish_video_track(&mut self, track_name: &str){
        if !self.is_multiplayer() {
            return;
        }

        if let Some(lk_service) = self.service.lock().as_mut(){
            let _ = lk_service.input_sender.send(RoomAction::PublishVideo { track_name: track_name.to_string() });
        }
    }

    pub fn is_multiplayer(&self) -> bool {
        if let Some(lk_service) = self.service.lock().as_ref() {
            if lk_service.connection_state.lock().eq(&ConnectionState::Connected) {
                return true;
            }
        }
        return false;
    }
}