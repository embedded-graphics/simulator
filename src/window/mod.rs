use std::{
    env,
    fs::File,
    io::BufReader,
    ops::Deref,
    process, thread,
    time::{Duration, Instant},
};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{
    display::SimulatorDisplay, output_image::OutputImage, output_settings::OutputSettings,
};

#[cfg(feature = "with-sdl")]
mod sdl_window;

#[cfg(feature = "with-sdl")]
pub use sdl_window::{SdlWindow, SimulatorEvent, SimulatorEventsIter};

#[cfg(feature = "with-sdl")]
mod multi_window;

#[cfg(feature = "with-sdl")]
pub use multi_window::MultiWindow;

pub(crate) struct FpsLimiter {
    max_fps: u32,
    frame_start: Instant,
}

impl FpsLimiter {
    pub(crate) fn new() -> Self {
        Self {
            max_fps: 60,
            frame_start: Instant::now(),
        }
    }

    fn desired_loop_duration(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.max_fps as f32)
    }

    fn sleep(&mut self) {
        let sleep_duration = (self.frame_start + self.desired_loop_duration())
            .saturating_duration_since(Instant::now());
        thread::sleep(sleep_duration);

        self.frame_start = Instant::now();
    }
}

/// Simulator window
#[allow(dead_code)]
pub struct Window {
    framebuffer: Option<OutputImage<Rgb888>>,
    #[cfg(feature = "with-sdl")]
    sdl_window: Option<SdlWindow>,
    title: String,
    output_settings: OutputSettings,
    fps_limiter: FpsLimiter,
}

impl Window {
    /// Creates a new simulator window.
    pub fn new(title: &str, output_settings: &OutputSettings) -> Self {
        Self {
            framebuffer: None,
            #[cfg(feature = "with-sdl")]
            sdl_window: None,
            title: String::from(title),
            output_settings: output_settings.clone(),
            fps_limiter: FpsLimiter::new(),
        }
    }

    /// Updates the window.
    pub fn update<C>(&mut self, display: &SimulatorDisplay<C>)
    where
        C: PixelColor + Into<Rgb888> + From<Rgb888>,
    {
        if let Ok(path) = env::var("EG_SIMULATOR_CHECK") {
            let output = display.to_rgb_output_image(&self.output_settings);

            let png_file = BufReader::new(File::open(path).unwrap());
            let expected = image::load(png_file, image::ImageFormat::Png)
                .unwrap()
                .to_rgb8();

            let png_size = Size::new(expected.width(), expected.height());

            assert!(
                output.size().eq(&png_size),
                "display dimensions don't match PNG dimensions (display: {}x{}, PNG: {}x{})",
                output.size().width,
                output.size().height,
                png_size.width,
                png_size.height
            );

            assert!(
                output
                    .as_image_buffer()
                    .as_raw()
                    .eq(&expected.as_raw().deref()),
                "display content doesn't match PNG file",
            );

            process::exit(0);
        }

        if let Ok(path) = env::var("EG_SIMULATOR_CHECK_RAW") {
            let expected = SimulatorDisplay::load_png(path).unwrap();

            assert!(
                display.size().eq(&expected.size()),
                "display dimensions don't match PNG dimensions (display: {}x{}, PNG: {}x{})",
                display.size().width,
                display.size().height,
                expected.size().width,
                expected.size().height
            );

            assert!(
                display.pixels.eq(&expected.pixels),
                "display content doesn't match PNG file",
            );

            process::exit(0);
        }

        if let Ok(path) = env::var("EG_SIMULATOR_DUMP") {
            display
                .to_rgb_output_image(&self.output_settings)
                .save_png(path)
                .unwrap();
            process::exit(0);
        }

        if let Ok(path) = env::var("EG_SIMULATOR_DUMP_RAW") {
            display
                .to_rgb_output_image(&OutputSettings::default())
                .save_png(path)
                .unwrap();
            process::exit(0);
        }

        #[cfg(feature = "with-sdl")]
        {
            let size = display.output_size(&self.output_settings);

            if self.framebuffer.is_none() {
                self.framebuffer = Some(OutputImage::new(size));
            }

            if self.sdl_window.is_none() {
                self.sdl_window = Some(SdlWindow::new(&self.title, size));
            }

            let framebuffer = self.framebuffer.as_mut().unwrap();
            let sdl_window = self.sdl_window.as_mut().unwrap();

            framebuffer.draw_display(display, Point::zero(), &self.output_settings);
            sdl_window.update(framebuffer);
        }

        self.fps_limiter.sleep();
    }

    /// Shows a static display.
    ///
    /// This methods updates the window once and loops until the simulator window
    /// is closed.
    pub fn show_static<C>(&mut self, display: &SimulatorDisplay<C>)
    where
        C: PixelColor + Into<Rgb888> + From<Rgb888>,
    {
        self.update(display);

        #[cfg(feature = "with-sdl")]
        'running: loop {
            if self.events().any(|e| e == SimulatorEvent::Quit) {
                break 'running;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    /// Returns an iterator of all captured simulator events.
    ///
    /// # Panics
    ///
    /// Panics if called before [`update`](Self::update) is called at least
    /// once. Also panics if multiple instances of the iterator are used at the
    /// same time.
    #[cfg(feature = "with-sdl")]
    pub fn events(&self) -> SimulatorEventsIter<'_> {
        self.sdl_window
            .as_ref()
            .unwrap()
            .events(&self.output_settings)
    }

    /// Sets the FPS limit of the window.
    pub fn set_max_fps(&mut self, max_fps: u32) {
        self.fps_limiter.max_fps = max_fps;
    }
}
