use crate::frame_streamer::{
    AudioStreamerMetaData, VideoFrame, VideoStreamer, VideoStreamerMetaData,
};
use ffmpeg::{
    self, channel_layout, codec, decoder,
    format::{context::Input, input, Pixel, Sample},
    frame::Audio,
    software::{
        resampling,
        scaling::{flag::Flags, Context},
    },
    util::frame::Video,
    Error, Packet,
};
use std::{collections::VecDeque, path::PathBuf, time::Duration};

const SAMPLE_RATE: u32 = 51200;

pub struct LocalVideo {
    #[allow(unused)]
    file_name: &'static str,
    current_frame: usize,
    pub metadata: VideoStreamerMetaData,
    pub audio_metadata: AudioStreamerMetaData,
    ictx: Input,
    video_decoder: decoder::Video,
    audio_decoder: decoder::Audio,
    video_stream_index: usize,
    audio_stream_index: usize,
    scaler: Context,
    resampler: resampling::Context,
    video_packet_buffer: VecDeque<Packet>,
    audio_packet_buffer: VecDeque<Packet>,
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
            codec::context::Context::from_parameters(audio_input.parameters()).unwrap();
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
        let resampler = resampling::Context::get(
            audio_decoder.format(),
            audio_decoder.channel_layout(),
            audio_decoder.rate(),
            Sample::F32(ffmpeg::format::sample::Type::Packed),
            channel_layout::ChannelLayout::MONO,
            SAMPLE_RATE,
        )
        .unwrap();

        let current_frame = 0;

        LocalVideo {
            file_name,
            current_frame,
            metadata: VideoStreamerMetaData {
                fps: yarden::get_fps(file_name),
                frame_width: yarden::get_width(file_name),
                frame_height: yarden::get_height(file_name),
            },
            audio_metadata: AudioStreamerMetaData {
                sample_rate: SAMPLE_RATE,
            },
            ictx,
            video_decoder,
            audio_decoder,
            video_stream_index,
            audio_stream_index,
            scaler,
            resampler,
            video_packet_buffer: VecDeque::new(),
            audio_packet_buffer: VecDeque::new(),
        }
    }

    pub fn buffer_packets(&mut self) {
        for (i, (stream, packet)) in self.ictx.packets().enumerate() {
            if stream.index() == self.audio_stream_index {
                self.audio_packet_buffer.push_back(packet);
            } else if stream.index() == self.video_stream_index {
                self.video_packet_buffer.push_back(packet);
            }
            if i > 10 {
                break;
            }
        }
    }

    pub fn poll_next_frame(&mut self) -> Option<Video> {
        if let Some(packet) = self.video_packet_buffer.pop_front() {
            self.video_decoder.send_packet(&packet).unwrap();
        } else {
            self.buffer_packets();
            return self.poll_next_frame();
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

    pub fn _poll_audio(&mut self) -> Option<SampleIter> {
        if let Some(packet) = self.audio_packet_buffer.pop_front() {
            self.audio_decoder.send_packet(&packet).unwrap();
        } else {
            self.buffer_packets();
            return self._poll_audio();
        }
        let mut decoded = Audio::empty();
        self.audio_decoder.receive_frame(&mut decoded).ok()?;
        let mut resampled = Audio::empty();
        self.resampler.run(&decoded, &mut resampled).unwrap();
        debug_assert_eq!(self.audio_metadata.sample_rate, resampled.rate());

        Some(SampleIter {
            audio: resampled,
            sample_idx: 0,
        })
    }
}

impl VideoFrame for Video {
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

impl VideoStreamer for LocalVideo {
    type Frame = Video;
    fn metadata(&self) -> VideoStreamerMetaData {
        self.metadata
    }
    fn next_frame(&mut self) -> Option<Self::Frame> {
        self.poll_next_frame()
    }
}

pub struct SampleIter {
    audio: Audio,
    sample_idx: usize,
}

impl rodio::Source for SampleIter {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.audio.samples())
    }
    fn sample_rate(&self) -> u32 {
        self.audio.rate()
    }
    fn channels(&self) -> u16 {
        self.audio.channels()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(Duration::from_nanos(
            self.audio.samples() as u64 * 1_000_000_000 / self.sample_rate() as u64,
        ))
    }
}

impl Iterator for SampleIter {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.audio.plane(0).get(self.sample_idx).copied();
        self.sample_idx += 1;
        x
    }
}
