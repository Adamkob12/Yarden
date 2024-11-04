pub trait FrameStreamer {
    type Frame;
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

pub struct FrameStreamerMetaData {
    pub fps: usize,
    pub frame_width: u32,
    pub frame_height: u32,
}

pub trait Frame {
    type Pixel;
    fn pixels(&self) -> impl Iterator<Item = (u32, u32, Self::Pixel)>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
