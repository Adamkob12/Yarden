// mod frame_fetcher;
// mod video_local;

use ffmpeg_sidecar::command::{ffmpeg_is_installed, FfmpegCommand};
use ffmpeg_sidecar::download::{
    check_latest_version, download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg,
};
use ffmpeg_sidecar::event::FfmpegEvent;
use ffmpeg_sidecar::iter::FfmpegIterator;
use ffmpeg_sidecar::paths::sidecar_dir;
use ffmpeg_sidecar::version::ffmpeg_version_with_path;
use softbuffer::{Context, Surface};
use std::env::current_exe;
use std::num::NonZeroU32;
use std::path::{Component, PathBuf};
use std::rc::Rc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use yarden::{get_fps, get_height, get_width};

struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    start_time: Instant,
    prev_frame: Instant,
    width: u32,
    height: u32,
    fps: usize,
    frames: FfmpegIterator,
    pixel_format: PixelFormat,
}

enum PixelFormat {
    ZeroRGB, // 00000000-RRRRRRRR-GGGGGGGG-BBBBBBBB
    Rgb24,   // RRRRRRRR-GGGGGGGG-BBBBBBBB
}

impl App {
    pub fn new(video_path: &'static str) -> App {
        let width = get_width(video_path);
        let height = get_height(video_path);
        let fps = get_fps(video_path);
        App {
            window: None,
            context: None,
            surface: None,
            start_time: Instant::now(),
            prev_frame: Instant::now(),
            frames: FfmpegCommand::new()
                .input("assets/test1.mp4")
                .args(["-f", "rawvideo"])
                .args(["-pix_fmt", "rgb0", "-"])
                .spawn()
                .unwrap()
                .iter()
                .unwrap(),
            width,
            height,
            fps,
            pixel_format: PixelFormat::ZeroRGB,
        }
    }

    #[allow(dead_code)]
    pub fn new_test() -> App {
        App {
            window: None,
            context: None,
            surface: None,
            start_time: Instant::now(),
            prev_frame: Instant::now(),
            frames: FfmpegCommand::new()
                .testsrc()
                .rawvideo()
                .spawn()
                .unwrap()
                .iter()
                .unwrap(),
            width: 320,
            height: 240,
            fps: 25,
            pixel_format: PixelFormat::Rgb24,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height)),
                )
                .unwrap(),
        );
        let context = Context::new(Rc::clone(&window)).unwrap();
        let mut surface = Surface::new(&context, Rc::clone(&window)).unwrap();

        surface
            .resize(
                NonZeroU32::new(self.width).unwrap(),
                NonZeroU32::new(self.height).unwrap(),
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
                while Instant::now().duration_since(self.prev_frame).as_millis()
                    < (1000 / self.fps as u128)
                {}
                self.prev_frame = Instant::now();
                let _width = self.width as usize;
                let _height = self.height as usize;

                // println!("Redraw at {:#?}", self.start_time.elapsed(),);

                let mut buff = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                while let Some(ffmpeg_event) = self.frames.next() {
                    match ffmpeg_event {
                        FfmpegEvent::OutputFrame(frame) => {
                            debug_assert_eq!(frame.width, self.width);
                            debug_assert_eq!(frame.height, self.height);
                            match self.pixel_format {
                                PixelFormat::ZeroRGB => unsafe {
                                    core::ptr::copy_nonoverlapping(
                                        frame.data.as_ptr(),
                                        buff.as_mut_ptr() as *mut u8,
                                        frame.data.len(),
                                    );
                                },
                                PixelFormat::Rgb24 => {
                                    for (i, rgb) in frame.data.chunks(3).enumerate() {
                                        let red = rgb[0] as u32;
                                        let green = rgb[1] as u32;
                                        let blue = rgb[2] as u32;
                                        buff[i] = blue | (green << 8) | (red << 16);
                                    }
                                }
                            }

                            break;
                        }
                        FfmpegEvent::Progress(progress) => {
                            eprintln!("Current speed: {}x", progress.speed); // <- parsed progress updates
                        }
                        FfmpegEvent::Log(_level, msg) => {
                            eprintln!("[ffmpeg] {}", msg); // <- granular log message from stderr
                        }
                        FfmpegEvent::ParsedInputStream(stream) => {
                            if let Some(video_data) = stream.video_data() {
                                println!(
                              "Found video stream with index {} in input {} that has fps {}, width {}px, height {}px.",
                              stream.stream_index,
                              stream.parent_index,
                              video_data.fps,
                              video_data.width,
                              video_data.height
                            );
                            }
                        }
                        x => {
                            eprintln!("Unhandled ffmpeg event: {:#?}", x);
                        }
                    }
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

    let mut app = App::new("assets/test1.mp4");
    event_loop.run_app(&mut app).unwrap();
}

fn ffmpeg_() {
    if ffmpeg_is_installed() {
        println!("FFmpeg is already installed! 🎉");
        println!("For demo purposes, we'll re-download and unpack it anyway.");
        println!("TIP: Use `auto_download()` to skip manual customization.");
    }

    // Short version without customization:
    // ```rust
    // ffmpeg_sidecar::download::auto_download().unwrap();
    // ```

    // Checking the version number before downloading is actually not necessary,
    // but it's a good way to check that the download URL is correct.
    match check_latest_version() {
        Ok(version) => println!("Latest available version: {}", version),
        Err(_) => println!("Skipping version check on this platform."),
    }

    // These defaults will automatically select the correct download URL for your
    // platform.
    let download_url = ffmpeg_download_url().unwrap();
    let cli_arg = std::env::args().nth(1);
    let destination = match cli_arg {
        Some(arg) => resolve_relative_path(current_exe().unwrap().parent().unwrap().join(arg)),
        None => sidecar_dir().unwrap(),
    };

    // The built-in download function uses `reqwest` to download the package.
    // For more advanced use cases like async streaming or download progress
    // updates, you could replace this with your own download function.
    println!("Downloading from: {:?}", download_url);
    let archive_path = download_ffmpeg_package(download_url, &destination).unwrap();
    println!("Downloaded package: {:?}", archive_path);

    // Extraction uses `tar` on all platforms (available in Windows since version 1803)
    println!("Extracting...");
    unpack_ffmpeg(&archive_path, &destination).unwrap();

    // Use the freshly installed FFmpeg to check the version number
    let version = ffmpeg_version_with_path(destination.join("ffmpeg")).unwrap();
    println!("FFmpeg version: {}", version);

    println!("Done! 🏁");
}

fn resolve_relative_path(path_buf: PathBuf) -> PathBuf {
    let mut components: Vec<PathBuf> = vec![];
    for component in path_buf.as_path().components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                components.push(component.as_os_str().into())
            }
            Component::CurDir => (),
            Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                }
            }
            Component::Normal(component) => components.push(component.into()),
        }
    }
    PathBuf::from_iter(components)
}
