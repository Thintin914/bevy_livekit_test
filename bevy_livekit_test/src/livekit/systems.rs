use bevy::prelude::*;
use livekit::RoomEvent;

use super::{service::ConnectionState, LKResource};

pub fn on_room_event_received(mut lk_resource: ResMut<LKResource>){
    if let Some(lk_service) = &lk_resource.service.lock().as_ref() {
        if let Some(event_receiver) = lk_service.event_receiver.lock().as_ref() {
            if let Ok(room_event) = event_receiver.try_recv() {
                match room_event {
                    _ => println!("{:?}", room_event)
                }
            }
        }
    }
}