use crate::frame_streamer::{Frame, FrameStreamer, FrameStreamerMetaData};
use ffmpeg::{
    self, codec, decoder,
    format::{context::Input, input, Pixel},
    frame::Audio,
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
    video_decoder: decoder::Video,
    audio_decoder: decoder::Audio,
    video_stream_index: usize,
    audio_stream_index: usize,
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
        let video_input = ictx
            .streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(Error::StreamNotFound)
            .unwrap();
        let audio_input = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .ok_or(Error::StreamNotFound)
            .unwrap();
        let video_stream_index = video_input.index();
        let audio_stream_index = audio_input.index();
        let video_context_decoder =
            codec::context::Context::from_parameters(video_input.parameters()).unwrap();
        let audio_context_decoder =
            codec::context::Context::from_parameters(video_input.parameters()).unwrap();
        let video_decoder = video_context_decoder.decoder().video().unwrap();
        let audio_decoder = audio_context_decoder.decoder().audio().unwrap();

        let scaler = Context::get(
            video_decoder.format(),
            video_decoder.width(),
            video_decoder.height(),
            Pixel::BGRZ,
            video_decoder.width(),
            video_decoder.height(),
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
            video_decoder,
            audio_decoder,
            video_stream_index,
            audio_stream_index,
        }
    }

    pub fn poll_next_frame(&mut self) -> Option<Video> {
        for (stream, packet) in self.ictx.packets() {
            if stream.index() == self.video_stream_index {
                self.video_decoder.send_packet(&packet).unwrap();
                break;
            }
        }
        let mut decoded = Video::empty();
        self.video_decoder.receive_frame(&mut decoded).ok()?;
        debug_assert_eq!(decoded.width(), self.metadata.frame_width);
        debug_assert_eq!(decoded.height(), self.metadata.frame_height);
        let mut rgb_frame = Video::empty();
        self.scaler.run(&decoded, &mut rgb_frame).unwrap();
        self.current_frame += 1;

        Some(rgb_frame)
    }

    pub fn _poll_audio(&mut self) -> Option<Audio> {
        for (stream, packet) in self.ictx.packets() {
            if stream.index() == self.audio_stream_index {
                self.audio_decoder.send_packet(&packet).unwrap();
                break;
            }
        }
        let mut decoded = Audio::empty();
        self.audio_decoder.receive_frame(&mut decoded).ok()?;
        println!(
            "Recieved {} samples at {} sample rate.",
            decoded.samples(),
            decoded.rate()
        );

        Some(decoded)
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
        self.poll_next_frame()
    }
}
