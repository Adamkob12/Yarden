use crate::frame_fetcher::FrameStreamerMetaData;
use std::path::PathBuf;

pub struct LocalVideo {
    file_name: &'static str,
    current_frame: usize,
    metadata: FrameStreamerMetaData,
}

impl LocalVideo {
    pub fn new(file_name: &'static str) -> LocalVideo {
        assert!(
            PathBuf::from(file_name).exists(),
            "{:#?} doesn't exist.",
            file_name
        );
        LocalVideo {
            file_name,
            current_frame: 0,
            metadata: FrameStreamerMetaData {
                fps: yarden::get_fps(file_name),
                frame_width: yarden::get_width(file_name),
                frame_height: yarden::get_height(file_name),
            },
        }
    }
}
