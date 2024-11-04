use image::{load_from_memory, DynamicImage, GenericImageView, Rgba};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    start_time: Instant,
    prev_frame: Instant,
    image: DynamicImage,
}

impl App {
    pub fn new() -> App {
        App {
            window: None,
            context: None,
            surface: None,
            start_time: Instant::now(),
            prev_frame: Instant::now(),
            image: load_from_memory(include_bytes!("../assets/fruit.jpg")).unwrap(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(
                    winit::dpi::PhysicalSize::new(self.image.width(), self.image.height()),
                ))
                .unwrap(),
        );
        let context = Context::new(Rc::clone(&window)).unwrap();
        let mut surface = Surface::new(&context, Rc::clone(&window)).unwrap();

        surface
            .resize(
                NonZeroU32::new(self.image.width()).unwrap(),
                NonZeroU32::new(self.image.height()).unwrap(),
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
            WindowEvent::RedrawRequested => {
                while Instant::now().duration_since(self.prev_frame).as_millis() < (1000 / 30) {}
                self.prev_frame = Instant::now();
                let width = self.image.width() as usize;
                let _height = self.image.height() as usize;

                let offset = self.start_time.elapsed().as_millis() as usize / 100;
                let offset_x = |x: u32| (x as usize + offset) % width;
                println!(
                    "Redraw at {:#?}, offset={}",
                    self.start_time.elapsed(),
                    offset
                );

                let mut buff = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                for (x, y, pixel) in self.image.pixels() {
                    buff[y as usize * width + offset_x(x)] = rgba_to_zrgb(pixel);
                }
                buff.present().unwrap();
                //
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    // This is ideal for non-game applications that only update in response to user
    // input, and uses significantly less power/CPU time than ControlFlow::Poll.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

/// RGBA: RRRRRRRR-GGGGGGGG-BBBBBBBB-AAAAAAAA
/// zrgb: 00000000-RRRRRRRR-GGGGGGGG-BBBBBBBB
fn rgba_to_zrgb(rgba: Rgba<u8>) -> u32 {
    let red = rgba.0[0] as u32;
    let green = rgba.0[1] as u32;
    let blue = rgba.0[2] as u32;

    blue | (green << 8) | (red << 16)

    // let tmp: u32 = unsafe { transmute(rgba) };
    // tmp >> 8
}
