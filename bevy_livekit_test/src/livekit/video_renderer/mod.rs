use async_std::stream::StreamExt;
use image::{ImageBuffer, Rgba};
use livekit::webrtc::native::yuv_helper;
use livekit::webrtc::prelude::*;
use livekit::webrtc::video_stream::native::NativeVideoStream;
use parking_lot::Mutex;
use std::{ops::DerefMut, sync::Arc};

pub struct VideoRenderer {
    internal: Arc<Mutex<RendererInternal>>,

    #[allow(dead_code)]
    rtc_track: RtcVideoTrack,

    stop_sender: flume::Sender<bool>
}

struct RendererInternal {
    width: u32,
    height: u32,
    rgba_data: Vec<u8>
}

impl VideoRenderer {
    pub fn new(
        rtc_track: RtcVideoTrack,
    ) -> Self {

        let (stop_sender, stop_receiver) = flume::bounded::<bool>(1);

        let internal = Arc::new(Mutex::new(RendererInternal {
            width: 0,
            height: 0,
            rgba_data: Vec::default()
        }));
        
        let mut video_sink = NativeVideoStream::new(rtc_track.clone());

        std::thread::spawn({
            let internal = internal.clone();
            move || {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    loop {
                        tokio::select! {
                            frame = video_sink.next() => {
                                if let Some(frame) = frame {
                                    // Process the frame
                                    let mut internal = internal.lock();
                                    let buffer = frame.buffer.to_i420();
            
                                    let width: u32 = buffer.width();
                                    let height: u32 = buffer.height();
            
                                    internal.ensure_texture_size(width, height);
            
                                    let rgba_ptr = internal.rgba_data.deref_mut();
                                    let rgba_stride = buffer.width() * 4;
            
                                    let (stride_y, stride_u, stride_v) = buffer.strides();
                                    let (data_y, data_u, data_v) = buffer.data();
            
                                    yuv_helper::i420_to_abgr(
                                        data_y,
                                        stride_y,
                                        data_u,
                                        stride_u,
                                        data_v,
                                        stride_v,
                                        rgba_ptr,
                                        rgba_stride,
                                        buffer.width() as i32,
                                        buffer.height() as i32,
                                    );
            
                                    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba_ptr.to_vec())
                                    .expect("Failed to create image buffer");

                                    let debug_path = std::path::PathBuf::from("src/debug").join("1.png");
                                    image.save(debug_path).expect("Failed to save debug image");
                                }
                            }
                            Ok(stop) = stop_receiver.recv_async() => {
                                if stop {
                                    println!("stop video renderer");
                                    break;
                                }
                            }
                        }
                    }
                });
            }
        });

        Self {
            rtc_track,
            internal,
            stop_sender
        }
    }

    pub fn stop(&self) {
        let _ = self.stop_sender.send(true);
    }

    // Returns the last frame resolution
    pub fn resolution(&self) -> (u32, u32) {
        let internal = self.internal.lock();
        (internal.width, internal.height)
    }

    pub fn rgba_data(&self) -> Vec<u8> {
        let internal = self.internal.lock();
        return internal.rgba_data.clone();
    }
}

impl RendererInternal {
    fn ensure_texture_size(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;
        self.rgba_data.resize((width * height * 4) as usize, 0);
    }
}