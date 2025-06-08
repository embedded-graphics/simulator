use std::collections::HashMap;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

use crate::{
    window::{sdl_window::SimulatorEventsIter, FpsLimiter, SdlWindow},
    OutputImage, OutputSettings, SimulatorDisplay,
};

/// Simulator window with support for multiple displays.
///
/// Multiple [`SimulatorDisplay`]s can be added to the window by using the
/// [`add_display`](Self::add_display) method. To update the window two steps
/// are required, first [`update_display`](Self::update_display) needs be called
/// for all changed displays, then [`flush`](Self::flush) to redraw the window.
///
/// To determine if the mouse pointer is over one of the displays the
/// [`translate_mouse_position`](Self::translate_mouse_position) can be used to
/// translate window coordinates into display coordinates.
pub struct MultiWindow {
    sdl_window: SdlWindow,
    framebuffer: OutputImage<Rgb888>,
    displays: HashMap<usize, DisplaySettings>,
    fps_limiter: FpsLimiter,
}

impl MultiWindow {
    /// Creates a new window with support for multiple displays.
    pub fn new(title: &str, size: Size) -> Self {
        let mut sdl_window = SdlWindow::new(title, size);

        let framebuffer = OutputImage::new(size);

        sdl_window.update(&framebuffer);

        Self {
            sdl_window: sdl_window,
            framebuffer,
            displays: HashMap::new(),
            fps_limiter: FpsLimiter::new(),
        }
    }

    /// Adds a display to the window.
    pub fn add_display<C>(
        &mut self,
        display: &SimulatorDisplay<C>,
        offset: Point,
        output_settings: &OutputSettings,
    ) {
        self.displays.insert(
            display.id,
            DisplaySettings {
                offset,
                output_settings: output_settings.clone(),
            },
        );
    }

    /// Fills the internal framebuffer with the given color.
    ///
    /// This method can be used to set the background color for the regions of
    /// the window that aren't covered by a display.
    pub fn clear(&mut self, color: Rgb888) {
        self.framebuffer.clear(color).unwrap();
    }

    /// Updates one display.
    ///
    /// This method only updates the internal framebuffer. Use
    /// [`flush`](Self::flush) after all displays have been updated to finally
    /// update the window.
    pub fn update_display<C>(&mut self, display: &SimulatorDisplay<C>)
    where
        C: PixelColor + Into<Rgb888> + From<Rgb888>,
    {
        let display_settings = self
            .displays
            .get(&display.id)
            .expect("update_display called for a display that hasn't been added with add_display");

        self.framebuffer.draw_display(
            display,
            display_settings.offset,
            &display_settings.output_settings,
        );
    }

    /// Updates the window from the internal framebuffer.
    pub fn flush(&mut self) {
        self.sdl_window.update(&self.framebuffer);

        self.fps_limiter.sleep();
    }

    /// Returns an iterator of all captured simulator events.
    ///
    /// The coordinates in mouse events are in raw window coordinates, use
    /// [`translate_mouse_position`](Self::translate_mouse_position) to
    /// translate them into display coordinates.
    ///
    /// # Panics
    ///
    /// Panics if multiple instances of the iterator are used at the same time.
    pub fn events(&self) -> SimulatorEventsIter<'_> {
        self.sdl_window.events(&crate::OutputSettings::default())
    }

    /// Translate a mouse position into display coordinates.
    ///
    /// Returns the corresponding position in the display coordinate system if
    /// the mouse is inside the display area, otherwise `None` is returned.
    pub fn translate_mouse_position<C>(
        &self,
        display: &SimulatorDisplay<C>,
        position: Point,
    ) -> Option<Point> {
        let display_settings = self.displays.get(&display.id).expect(
            "translate_mouse_position called for a display that hasn't been added with add_display",
        );

        let delta = position - display_settings.offset;
        let p = display_settings.output_settings.output_to_display(delta);

        display.bounding_box().contains(p).then_some(p)
    }

    /// Sets the FPS limit of the window.
    pub fn set_max_fps(&mut self, max_fps: u32) {
        self.fps_limiter.max_fps = max_fps;
    }
}

struct DisplaySettings {
    offset: Point,
    output_settings: OutputSettings,
}
