use crate::frame_streamer::{Frame, FrameStreamer, FrameStreamerMetaData};
use ffmpeg::{
    self, codec, decoder,
    format::{context::Input, input, Pixel},
    software::scaling::{flag::Flags, Context},
    util::frame::Video,
    Error,
};
use std::path::PathBuf;

pub struct LocalVideo {
    #[allow(unused)]
    file_name: &'static str,
    current_frame: usize,
    pub metadata: FrameStreamerMetaData,
    ictx: Input,
    decoder: decoder::Video,
    video_stream_index: usize,
    scaler: Context,
}

impl LocalVideo {
    pub fn new(file_name: &'static str) -> LocalVideo {
        assert!(
            PathBuf::from(file_name).exists(),
            "{:#?} doesn't exist.",
            file_name
        );
        let ictx = input(file_name).unwrap();
        let input = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(Error::StreamNotFound)
            .unwrap();
        let video_stream_index = input.index();
        let context_decoder = codec::context::Context::from_parameters(input.parameters()).unwrap();
        let decoder = context_decoder.decoder().video().unwrap();

        let scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::BGRZ,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )
        .unwrap();

        let current_frame = 0;

        LocalVideo {
            scaler,
            file_name,
            current_frame,
            metadata: FrameStreamerMetaData {
                fps: yarden::get_fps(file_name),
                frame_width: yarden::get_width(file_name),
                frame_height: yarden::get_height(file_name),
            },
            ictx,
            decoder,
            video_stream_index,
        }
    }

    pub fn next_frame(&mut self) -> Option<Video> {
        for (stream, packet) in self.ictx.packets() {
            if stream.index() == self.video_stream_index {
                self.decoder.send_packet(&packet).unwrap();
                break;
            }
        }
        let mut decoded = Video::empty();
        self.decoder.receive_frame(&mut decoded).ok()?;
        debug_assert_eq!(decoded.width(), self.metadata.frame_width);
        debug_assert_eq!(decoded.height(), self.metadata.frame_height);
        let mut rgb_frame = Video::empty();
        self.scaler.run(&decoded, &mut rgb_frame).unwrap();
        self.current_frame += 1;

        Some(rgb_frame)
    }
}

impl Frame for Video {
    fn width(&self) -> u32 {
        self.width()
    }
    fn height(&self) -> u32 {
        self.height()
    }
    fn bgrz_pixels(&self) -> &[u8] {
        debug_assert_eq!(self.format(), Pixel::BGRZ);
        self.data(0)
    }
}

impl FrameStreamer for LocalVideo {
    type Frame = Video;
    fn metadata(&self) -> FrameStreamerMetaData {
        self.metadata
    }
    fn next_frame(&mut self) -> Option<Self::Frame> {
        self.next_frame()
    }
}
