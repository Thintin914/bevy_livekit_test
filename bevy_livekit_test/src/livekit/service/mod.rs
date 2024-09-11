use super::video;

use std::{collections::HashMap, env, sync::Arc};

use image::RgbaImage;
use livekit::{Room, RoomEvent, RoomOptions};
use livekit_api::{access_token, services::room::RoomClient};
use parking_lot::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use video::DeviceVideoTrack;

pub struct LKService {
    room: Arc<Mutex<Option<Room>>>,
    pub connection_state: Arc<Mutex<ConnectionState>>,
    pub input_sender: flume::Sender<RoomAction>,
    pub event_receiver: Arc<Mutex<Option<flume::Receiver<RoomEvent>>>>,
    stop_sender: Arc<Mutex<Option<flume::Sender<bool>>>>,

    pub local_video_tracks: Arc<Mutex<HashMap<String, DeviceVideoTrack>>>
}

#[derive(PartialEq, Eq)]
pub enum ConnectionState {
    NotConnected,
    Connecting,
    Connected
}

pub enum RoomAction {
    ConnectRoom {
        room_id: String,
        user_uuid: String
    },
    LeaveRoom {
        room_id: String,
        user_uuid: String
    },
    PublishVideo {
        track_name: String
    }
}

impl LKService {
    pub fn new() -> LKService {
        let (input_sender, input_receiver) = flume::bounded::<RoomAction>(1);

        let mut lk_service = LKService {
            room: Arc::new(Mutex::new(None)),
            connection_state: Arc::new(Mutex::new(ConnectionState::NotConnected)),
            input_sender: input_sender,
            event_receiver: Arc::new(Mutex::new(None)),
            stop_sender: Arc::new(Mutex::new(None)),
            local_video_tracks: Arc::new(Mutex::new(HashMap::new()))
        };

        lk_service.thread(input_receiver);
        return lk_service;
    }

    pub async fn create_room(room_id: &str, user_uuid: &str, room_arc: Arc<Mutex<Option<Room>>>, stop_sender_arc: Arc<Mutex<Option<flume::Sender<bool>>>>, event_receiver_arc: Arc<Mutex<Option<flume::Receiver<RoomEvent>>>>){        
        let (stop_sender, stop_receiver) = flume::bounded::<bool>(1);
        let (event_sender, event_receiver) = flume::bounded::<RoomEvent>(1);

        let (url, api_key, api_secret, https_url) = Self::get_livekit_env();
        let (room, room_rx) = connect(&room_id, &user_uuid, &url, &api_key, &api_secret, true).await;

        *room_arc.lock() = Some(room);
        *stop_sender_arc.lock() = Some(stop_sender);
        *event_receiver_arc.lock() = Some(event_receiver);
        Self::event(room_rx, stop_receiver, event_sender);
    }

    fn thread(&mut self, input_receiver: flume::Receiver<RoomAction>){
        let (url, api_key, api_secret, https_url) = Self::get_livekit_env();
        let room_service = RoomClient::with_api_key(&https_url, &api_key, &api_secret);
        let stop_sender = self.stop_sender.clone();
        let event_receiver = self.event_receiver.clone();
        let room = self.room.clone();
        let connection_state = self.connection_state.clone();
        let local_video_tracks = Arc::clone(&self.local_video_tracks);

        std::thread::spawn(move || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                loop {
                    while let Ok(room_action) = input_receiver.recv() {
                        match room_action {
                            RoomAction::ConnectRoom { room_id, user_uuid } => {
                                *connection_state.lock() = ConnectionState::Connecting;
                                Self::create_room(&room_id, &user_uuid, room.clone(), stop_sender.clone(), event_receiver.clone()).await;
                                *connection_state.lock() = ConnectionState::Connected;
                            },
                            RoomAction::LeaveRoom { room_id, user_uuid } => {
                                if let Some(stop_sender) = stop_sender.lock().as_ref() {
                                    let _ = stop_sender.send(true);
                                }
                                let _ = room_service.remove_participant(&room_id, &user_uuid).await;
                                *connection_state.lock() = ConnectionState::NotConnected;
                                break;
                            },
                            RoomAction::PublishVideo { track_name } => {
                                if local_video_tracks.lock().contains_key(&track_name) {
                                    return;
                                }
                                
                                if room.lock().is_some() {
                                    let mut track = DeviceVideoTrack::new(room.clone());
                                    let _ = track.publish(&track_name).await;
                                    local_video_tracks.lock().insert(track_name, track);
                                }
                            },
                        }
                    }
                }
            });
        });
    }

    fn event(mut room_rx: UnboundedReceiver<RoomEvent>, stop_receiver: flume::Receiver<bool>, event_sender: flume::Sender<RoomEvent>) {    
        tokio::spawn(async move {
            async_std::task::block_on(async {
                loop {
                    if let Ok(stop) = stop_receiver.try_recv() {
                        if stop {
                            drop(stop_receiver);
                            drop(event_sender);    
                            println!("stop loop");
                            break;
                        }
                    }
            
                    if let Ok(room_event) = room_rx.try_recv() {
                        let _ = event_sender.send(room_event);
                    }
                }
            });
        });
    }

    fn get_livekit_env() -> (String, String, String, String){
        let url = env::var("LIVEKIT_URL").expect("LIVEKIT_URL is not set");
        let api_key = env::var("LIVEKIT_API_KEY").expect("LIVEKIT_API_KEY is not set");
        let api_secret = env::var("LIVEKIT_API_SECRET").expect("LIVEKIT_API_SECRET is not set");
    
        let mut https_url = url.to_string();
        if https_url.starts_with("wss") {
            https_url = https_url.replace("wss", "https");
        }
    
        return (url, api_key, api_secret, https_url);
    }
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