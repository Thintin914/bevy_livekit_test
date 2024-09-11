use image::ImageFormat;
use image::RgbaImage;
use livekit::options::TrackPublishOptions;
use livekit::prelude::*;
use livekit::webrtc::native::yuv_helper;
use livekit::webrtc::prelude::VideoBuffer;
use livekit::webrtc::video_source::RtcVideoSource;
use livekit::webrtc::video_source::VideoResolution;
use livekit::webrtc::{
    video_frame::{I420Buffer, VideoFrame, VideoRotation},
    video_source::native::NativeVideoSource,
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub const FB_WIDTH: usize = 512;
pub const FB_HEIGHT: usize = 256;

#[derive(Clone)]
struct FrameData {
    framebuffer: Arc<Mutex<Vec<u8>>>,
    video_frame: Arc<Mutex<VideoFrame<I420Buffer>>>,
}

struct TrackHandle {
    close_tx: flume::Sender<bool>,
    track: LocalVideoTrack,
}

pub struct DeviceVideoTrack {
    rtc_source: NativeVideoSource,
    room: Arc<Mutex<Option<Room>>>,
    handle: Option<TrackHandle>,
}

impl DeviceVideoTrack {
    pub fn new(room: Arc<Mutex<Option<Room>>>) -> Self {
        Self {
            rtc_source: NativeVideoSource::new(VideoResolution {
                width: FB_WIDTH as u32,
                height: FB_HEIGHT as u32,
            }),
            room,
            handle: None,
        }
    }

    pub async fn publish(&mut self, track_name: &str) {
        println!("try publish 1");
        self.unpublish().await;

        let (close_sender, close_receiver) = flume::bounded(1);
        let source = self.rtc_source.clone();
        let track = LocalVideoTrack::create_video_track(
            &track_name,
            RtcVideoSource::Native(source.clone()),
        );      

        println!("try publish 2");
        tokio::spawn(async move {
            Self::track_task(close_receiver, source.clone());
        });

        println!("try publish 3");
        if let Some(room) = self.room.lock().as_ref() {
            match room.local_participant().publish_track(LocalTrack::Video(track.clone()), TrackPublishOptions {
                source: TrackSource::Screenshare,
                ..Default::default()
            }).await {
                Ok(_) => println!("OK"),
                Err(e) => println!("{:?}", e),
            }
        }

        println!("try publish 4");

        let handle = TrackHandle {
            close_tx: close_sender,
            track,
        };

        self.handle = Some(handle);
    }

    pub async fn unpublish(&mut self) {
        if self.handle.is_none() {
            return;
        }

        if let Some(handle) = self.handle.take() {
            let _ = handle.close_tx.send(true);

            if let Some(room) = self.room.lock().as_ref() {
                let _ = room
                .local_participant()
                .unpublish_track(&handle.track.sid())
                .await;
            }
        }
    }

    fn track_task(close_receiver: flume::Receiver<bool>, source: NativeVideoSource) {
        println!("track task");
        let image = image::load_from_memory_with_format(include_bytes!("1.png"), ImageFormat::Png)
        .unwrap()
        .to_rgba8();

        let data = FrameData {
            framebuffer: Arc::new(Mutex::new(vec![0u8; FB_WIDTH * FB_HEIGHT * 4])),
            video_frame: Arc::new(Mutex::new(VideoFrame {
                rotation: VideoRotation::VideoRotation0,
                buffer: I420Buffer::new(FB_WIDTH as u32, FB_HEIGHT as u32),
                timestamp_us: 0,
            }))
        };
       
        let duration = Duration::from_millis(1000 / 15);
        loop {
            if let Ok(stop) = close_receiver.try_recv() {
                if stop {
                    break;
                }
            }
            // if let Ok(image) = image_receiver.try_recv() {
                let image_data = image.as_raw();
                let image_width = image.width();
                let image_stride = (image_width * 4) as usize;
                let image_height = image.height();
                
                let mut framebuffer = data.framebuffer.lock();
                let mut video_frame = data.video_frame.lock();

                if video_frame.buffer.width().ne(&image_width) || video_frame.buffer.height().ne(&image_height) {
                    framebuffer.resize(image_stride * image_height as usize, 0);
                    video_frame.buffer = I420Buffer::new(image_width, image_height);
                }
                
                let (stride_y, stride_u, stride_v) = video_frame.buffer.strides();
                let (data_y, data_u, data_v) = video_frame.buffer.data_mut();

                for y in 0..image_height as usize {
                    let img_start = y * image_stride;
                    framebuffer[img_start..img_start + image_stride]
                        .copy_from_slice(&image_data[img_start..img_start + image_stride]);
                }

                yuv_helper::abgr_to_i420(
                    &framebuffer,
                    image_stride as u32,
                    data_y,
                    stride_y,
                    data_u,
                    stride_u,
                    data_v,
                    stride_v,
                    image_width as i32,
                    image_height as i32,
                );

                source.capture_frame::<I420Buffer>(&video_frame);
            // }
            std::thread::sleep(duration);
        }
    }
}

impl Drop for DeviceVideoTrack {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.close_tx.send(true);
        }
    }
}