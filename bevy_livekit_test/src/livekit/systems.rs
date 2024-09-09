use bevy::prelude::*;

use super::RTCResource;

pub fn wait_room_created(mut rtc_resource: ResMut<RTCResource>) {
    if rtc_resource.is_multiplayer() {
        return;
    }

    if let Some(room_receiver) = &rtc_resource.room_receiver {
        if let Ok(room_arc) = room_receiver.try_recv() {
            println!("room set");
            rtc_resource.room = Some(room_arc);
        }
    }
}

pub fn on_event_received(rtc_resource: Res<RTCResource>){
    if let Some(room_event_receiver) = &rtc_resource.room_event {
        if let Ok(room_event) = room_event_receiver.try_recv() {
            println!("----------");
            match room_event {
                _ => {println!("{:?}", room_event)}
            }
        }
    }
}