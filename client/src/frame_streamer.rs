pub trait VideoStreamer {
    type Frame: VideoFrame;
    fn metadata(&self) -> VideoStreamerMetaData;
    fn next_frame(&mut self) -> Option<Self::Frame>;
    fn fps(&self) -> usize {
        self.metadata().fps
    }
    fn frame_width(&self) -> u32 {
        self.metadata().frame_width
    }
    fn frame_height(&self) -> u32 {
        self.metadata().frame_height
    }
}

#[derive(Clone, Copy)]
pub struct VideoStreamerMetaData {
    pub fps: usize,
    pub frame_width: u32,
    pub frame_height: u32,
}

#[derive(Clone, Copy)]
pub struct AudioStreamerMetaData {
    pub sample_rate: u32,
}

pub trait VideoFrame {
    /// Must output BGRZ
    fn bgrz_pixels(&self) -> &[u8];
    #[allow(unused)]
    fn width(&self) -> u32;
    #[allow(unused)]
    fn height(&self) -> u32;
}

// pub trait AudioFrame {
//     /// Must output BGRZ
//     fn samples(&self) -> &[u8];
//     #[allow(unused)]
//     fn samples(&self) -> usize;
//     #[allow(unused)]
//     fn sample_rate(&self) -> u32;
// }
