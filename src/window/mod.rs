use std::{fs::File, io::BufReader, ops::Deref};

#[cfg(feature = "with-sdl")]
use std::{thread, time::Duration};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{
    display::SimulatorDisplay, output_image::OutputImage, output_settings::OutputSettings,
};

#[cfg(feature = "with-sdl")]
mod sdl;

#[cfg(feature = "with-sdl")]
pub use sdl::{
    SimulatorEvent,
    SdlWindow,
    SdlWindowTexture,
};

/// Simulator window
#[allow(dead_code)]
pub struct Window {
    framebuffer: Option<OutputImage<Rgb888>>,
    #[cfg(feature = "with-sdl")]
    sdl_window: Option<SdlWindow>,
    title: String,
    output_settings: OutputSettings,
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
        }
    }

    /// Updates the window.
    pub fn update<C>(&mut self, display: &SimulatorDisplay<C>)
    where
        C: PixelColor + Into<Rgb888> + From<Rgb888>,
    {
        if let Ok(path) = std::env::var("EG_SIMULATOR_CHECK") {
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

            std::process::exit(0);
        }

        if let Ok(path) = std::env::var("EG_SIMULATOR_CHECK_RAW") {
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

            std::process::exit(0);
        }

        if let Ok(path) = std::env::var("EG_SIMULATOR_DUMP") {
            display
                .to_rgb_output_image(&self.output_settings)
                .save_png(path)
                .unwrap();
            std::process::exit(0);
        }

        if let Ok(path) = std::env::var("EG_SIMULATOR_DUMP_RAW") {
            display
                .to_rgb_output_image(&OutputSettings::default())
                .save_png(path)
                .unwrap();
            std::process::exit(0);
        }

        #[cfg(feature = "with-sdl")]
        {
            if self.framebuffer.is_none() {
                self.framebuffer = Some(OutputImage::new(display, &self.output_settings));
            }

            if self.sdl_window.is_none() {
                self.sdl_window = Some(SdlWindow::new(display, &self.title, &self.output_settings));
            }

            let framebuffer = self.framebuffer.as_mut().unwrap();
            let sdl_window = self.sdl_window.as_mut().unwrap();

            framebuffer.update(display);
            sdl_window.update(&framebuffer);
        }
    }

    /// Shows a static display.
    ///
    /// This methods updates the window once and loops until the simulator window
    /// is closed.
    pub fn show_static<C>(&mut self, display: &SimulatorDisplay<C>)
    where
        C: PixelColor + Into<Rgb888> + From<Rgb888>,
    {
        self.update(&display);

        #[cfg(feature = "with-sdl")]
        'running: loop {
            if self.events().any(|e| e == SimulatorEvent::Quit) {
                break 'running;
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    /// Returns an iterator of all captured SimulatorEvents.
    ///
    /// # Panics
    ///
    /// Panics if called before [`update`](Self::update) is called at least once.
    #[cfg(feature = "with-sdl")]
    pub fn events(&mut self) -> impl Iterator<Item = SimulatorEvent> + '_ {
        self.sdl_window
            .as_mut()
            .unwrap()
            .events(&self.output_settings)
    }
}

