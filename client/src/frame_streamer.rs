pub trait FrameStreamer {
    type Frame: Frame;
    fn metadata(&self) -> FrameStreamerMetaData;
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
pub struct FrameStreamerMetaData {
    pub fps: usize,
    pub frame_width: u32,
    pub frame_height: u32,
}

pub trait Frame {
    /// Must output BGRZ
    fn bgrz_pixels(&self) -> &[u8];
    #[allow(unused)]
    fn width(&self) -> u32;
    #[allow(unused)]
    fn height(&self) -> u32;
}
