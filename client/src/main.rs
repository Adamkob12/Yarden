mod frame_streamer;
mod video_local;

use frame_streamer::{VideoFrame, VideoStreamer};
use rodio::{OutputStream, Sink};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::thread::yield_now;
use std::time::Instant;
use video_local::LocalVideo;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use winit_input_helper::WinitInputHelper;

struct App<F: VideoStreamer> {
    window: Window,
    context: Option<Context>,
    surface: Option<Surface>,
    _start_time: Instant,
    prev_frame: Instant,
    frame_streamer: F,
    _stream: OutputStream,
    is_paused: bool,
    is_muted: bool,
    volume: f32,
    sink: Sink,
}

impl App<LocalVideo> {
    pub fn new_local(video_path: &'static str, window: Window) -> App<LocalVideo> {
        let frame_streamer = LocalVideo::new(video_path);
        let context = unsafe { softbuffer::Context::new(&window).unwrap() };
        let mut surface = unsafe { softbuffer::Surface::new(&context, &window).unwrap() };
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        surface
            .resize(
                NonZeroU32::new(frame_streamer.frame_width()).unwrap(),
                NonZeroU32::new(frame_streamer.frame_height()).unwrap(),
            )
            .unwrap();
        window.set_inner_size(PhysicalSize::new(
            frame_streamer.frame_width(),
            frame_streamer.frame_height(),
        ));
        App {
            window,
            context: Some(context),
            surface: Some(surface),
            _start_time: Instant::now(),
            prev_frame: Instant::now(),
            frame_streamer,
            _stream,
            is_paused: false,
            is_muted: false,
            volume: 1.0,
            sink: Sink::try_new(&stream_handle).unwrap(),
        }
    }
}

fn main() {
    let mut input = WinitInputHelper::new();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("Video Player");

    let mut app = App::new_local("assets/test1.mp4", window);
    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::RedrawRequested(_window_id) => {
                while Instant::now().duration_since(app.prev_frame).as_millis()
                    < (1000 / app.frame_streamer.fps() as u128)
                    || app.is_paused
                {
                    yield_now();
                    app.window.request_redraw();
                    return;
                }
                app.prev_frame = Instant::now();
                let mut buff = app.surface.as_mut().unwrap().buffer_mut().unwrap();

                let video_frame = loop {
                    if let Some(x) = app.frame_streamer.next_frame() {
                        break x;
                    }
                    yield_now();
                };

                let audio_source = app.frame_streamer._poll_audio().unwrap();
                let audio_source2 = app.frame_streamer._poll_audio().unwrap();

                let frame_data = video_frame.bgrz_pixels();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        frame_data.as_ptr(),
                        buff.as_mut_ptr() as *mut _,
                        frame_data.len(),
                    )
                };
                app.sink.append(audio_source);
                app.sink.append(audio_source2);
                buff.present().unwrap();
            }
            _ => {}
        }

        if input.update(&event) {
            return;
        }

        // app.window.request_redraw();
    });
}
