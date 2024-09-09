use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use image::RgbaImage;

use std::time::Instant;
use bevy::tasks::AsyncComputeTaskPool;
use flume::{bounded, Receiver, Sender};
use livekit::prelude::*;

use tokio::sync::mpsc::UnboundedReceiver;

use std::collections::HashMap;
use std::{env, sync::Arc};
use parking_lot::Mutex;

use livekit_api::{access_token, services::room::RoomClient};

use super::video::DeviceVideoTrack;
use super::video_renderer::VideoRenderer;

#[derive(Resource)]
pub struct RTCResource {
    pub room_receiver: Option<flume::Receiver<Arc<Room>>>,
    pub room: Option<Arc<Room>>,

    pub stop_room_sender: Option<flume::Sender<bool>>,
    pub room_event: Option<Receiver<livekit::RoomEvent>>,

    topic_cooldown: HashMap<String, (u128, Option<Instant>)>,

    local_video_tracks: Arc<Mutex<HashMap<String, DeviceVideoTrack>>>,
    published_video_tracks: HashMap<String, VideoRenderer>
}

impl RTCResource {

    pub fn new_room(&mut self, username: &str) {
        if self.room.is_some() {
            return;
        }
        let username = username.to_string();

        let (room_sender, room_receiver) = bounded::<Arc<Room>>(1);
        self.room_receiver = Some(room_receiver);
        
        let (stop_room_sender, stop_room_receiver) = bounded::<bool>(1);
        self.stop_room_sender = Some(stop_room_sender);

        let (room_data_sender, room_data_receiver) = bounded::<livekit::RoomEvent>(1);
        self.room_event = Some(room_data_receiver);
        
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {

                // let room_service = RoomClient::with_api_key(&"https://shalloville-9m4b7i8o.livekit.cloud", &"APIuTz9ZrTcAwHg", &"OOnQS9PSAnZgfnzKfuensuSy44LJCNOZpdxgEb17aBYB");
                let (room, rx) = Self::connect("test", &username, "wss://shalloville-9m4b7i8o.livekit.cloud", "APIuTz9ZrTcAwHg", "OOnQS9PSAnZgfnzKfuensuSy44LJCNOZpdxgEb17aBYB", true).await;

                let _ = room_sender.send(Arc::new(room));
                Self::event(rx, stop_room_receiver, room_data_sender).await;
            });
        });
    }

    pub fn leave_room(&mut self, user_uuid: &str) {
        if self.room.is_none() {
            return;
        }

        let user_uuid = user_uuid.to_string();
        let mut room_id = String::new();
        if let Some(room) = &self.room {
            room_id = room.name();
        }

        if let Some(stop_room_sender) = &self.stop_room_sender {
            let _ = stop_room_sender.send(true);
        }
        self.stop_room_sender = None;
        self.room = None;
        self.room_event = None;
        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _ = get_room_service().remove_participant(&room_id, &user_uuid).await;
            });
        });
    }

    pub fn is_multiplayer(&self) -> bool {
        if self.room.is_some() {
            return true;
        }
        return false;
    }

    pub fn new_video_track(&mut self, track_name: &str){
        if self.room.is_none(){
            return;
        }
        if self.local_video_tracks.lock().contains_key(track_name) {
            return;
        }

        if let Some(room) = &self.room {
            let track_name = track_name.to_string();
            let room = Arc::clone(&room);
            let video_tracks = Arc::clone(&self.local_video_tracks);
            std::thread::spawn( move || {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let mut video_track = DeviceVideoTrack::new(room.clone());
                    let _ = video_track.publish(&track_name).await;
                    video_tracks.lock().insert(track_name.to_string(), video_track);
                });
            });
        }
    }

    pub async fn close_video_track(&mut self, track_name: &str) {
        if !self.local_video_tracks.lock().contains_key(track_name) {
            return;
        }

        let cloned_track_name = track_name.to_string();
        let video_tracks_arc = Arc::clone(&self.local_video_tracks);
        std::thread::spawn( move || {
            let mut track_lock = video_tracks_arc.lock();
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                if let Some(video_track) = track_lock.get_mut(&cloned_track_name) {
                    video_track.unpublish().await;
                    track_lock.remove(&cloned_track_name);
                }
            });
        });
    }

    async fn event(mut room_rx: UnboundedReceiver<RoomEvent>, stop_room_receiver: Receiver<bool>, room_data_sender: Sender<livekit::RoomEvent>) {    
        async_std::task::block_on(async {
            loop {
                if let Ok(stop) = stop_room_receiver.try_recv() {
                    if stop {
                        drop(stop_room_receiver);
                        drop(room_data_sender);    
                        println!("stop loop");
                        break;
                    }
                }
        
                if let Ok(room_event) = room_rx.try_recv() {
                    let _ = room_data_sender.send(room_event);
                }
            }
        });
    }

    async fn connect(room_id: &str, user_uuid: &str, url: &str, api_key: &str, api_secret: &str, admin: bool) -> (livekit::Room, UnboundedReceiver<livekit::RoomEvent>) {
        let token = access_token::AccessToken::with_api_key(&api_key, &api_secret)
        .with_identity(&user_uuid)
        .with_name(&user_uuid)
        .with_grants(access_token::VideoGrants {
            room_join: true,
            room_admin: admin,
            room: room_id.to_string(),
            can_publish: true,
            can_publish_data: true,
            can_subscribe: true,
            can_update_own_metadata: true,
            ..Default::default()
        })
        .to_jwt()
        .unwrap();
    
        let (room, rx) = Room::connect(&url, &token, RoomOptions {
            auto_subscribe: true,
            ..Default::default()
        })
        .await
        .unwrap();
    
        return (room, rx);
    }
}

impl Default for RTCResource {
    fn default() -> RTCResource {
        RTCResource {
            room_receiver: None,
            room: None,
            stop_room_sender: None,
            room_event: None,

            topic_cooldown: HashMap::from([
                ("move".to_string(), (200, None))
            ]),

            local_video_tracks: Arc::new(Mutex::new(HashMap::new())),
            published_video_tracks: HashMap::new(),
        }
    }
}

fn get_room_service() -> RoomClient{
    let room_service = RoomClient::with_api_key(&"https://shalloville-9m4b7i8o.livekit.cloud", &"APIuTz9ZrTcAwHg", &"OOnQS9PSAnZgfnzKfuensuSy44LJCNOZpdxgEb17aBYB");
    return room_service;
}