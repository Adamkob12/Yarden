mod frame_streamer;
mod video_local;

use frame_streamer::{VideoFrame, VideoStreamer};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::thread::yield_now;
use std::time::Instant;
use video_local::LocalVideo;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

struct App<F: VideoStreamer> {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    _start_time: Instant,
    prev_frame: Instant,
    frame_streamer: F,
    // stream_handle: OutputStreamHandle,
    stream: OutputStream,
    is_paused: bool,
    sink: Sink,
}

impl App<LocalVideo> {
    pub fn new_local(video_path: &'static str) -> App<LocalVideo> {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        App {
            window: None,
            context: None,
            surface: None,
            _start_time: Instant::now(),
            prev_frame: Instant::now(),
            frame_streamer: LocalVideo::new(video_path),
            // stream_handle,
            stream,
            is_paused: false,
            sink: Sink::try_new(&stream_handle).unwrap(),
        }
    }
}

impl ApplicationHandler for App<LocalVideo> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(
                    winit::dpi::PhysicalSize::new(
                        self.frame_streamer.frame_width(),
                        self.frame_streamer.frame_height(),
                    ),
                ))
                .unwrap(),
        );
        let context = Context::new(Rc::clone(&window)).unwrap();
        let mut surface = Surface::new(&context, Rc::clone(&window)).unwrap();

        surface
            .resize(
                NonZeroU32::new(self.frame_streamer.frame_width()).unwrap(),
                NonZeroU32::new(self.frame_streamer.frame_height()).unwrap(),
            )
            .unwrap();

        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.repeat
                    && event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::Space)
                {
                    self.is_paused = !self.is_paused;
                }
            }
            WindowEvent::RedrawRequested => {
                while Instant::now().duration_since(self.prev_frame).as_millis()
                    < (1000 / self.frame_streamer.fps() as u128)
                    || self.is_paused
                {
                    yield_now();
                    self.window.as_ref().unwrap().request_redraw();
                    return;
                }
                self.prev_frame = Instant::now();

                let mut buff = self.surface.as_mut().unwrap().buffer_mut().unwrap();

                let video_frame = loop {
                    if let Some(x) = self.frame_streamer.next_frame() {
                        break x;
                    }
                    yield_now();
                };

                let audio_source = self.frame_streamer._poll_audio().unwrap();
                let audio_source2 = self.frame_streamer._poll_audio().unwrap();

                let frame_data = video_frame.bgrz_pixels();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        frame_data.as_ptr(),
                        buff.as_mut_ptr() as *mut _,
                        frame_data.len(),
                    )
                };
                self.sink.append(audio_source);
                self.sink.append(audio_source2);
                buff.present().unwrap();
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new_local("assets/test1.mp4");
    event_loop.run_app(&mut app).unwrap();
}
