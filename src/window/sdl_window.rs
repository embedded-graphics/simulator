use std::cell::{RefCell, RefMut};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{Point, Size},
};
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::{MouseButton, MouseWheelDirection},
    pixels::PixelFormatEnum,
    render::{Canvas, Texture, TextureCreator},
    video::WindowContext,
    EventPump,
};

use crate::{OutputImage, OutputSettings};

/// A derivation of [`sdl2::event::Event`] mapped to embedded-graphics coordinates
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SimulatorEvent {
    /// A keypress event, fired on keyUp
    KeyUp {
        /// The key being released
        keycode: Keycode,
        /// Any modifier being held at the time of keyup
        keymod: Mod,
        /// Whether the key is repeating
        repeat: bool,
    },
    /// A keypress event, fired on keyDown
    KeyDown {
        /// The key being pressed
        keycode: Keycode,
        /// Any modifier being held at the time of keydown
        keymod: Mod,
        /// Whether the key is repeating
        repeat: bool,
    },
    /// A mouse click event, fired on mouseUp
    MouseButtonUp {
        /// The mouse button being released
        mouse_btn: MouseButton,
        /// The location of the mouse in Simulator coordinates
        point: Point,
    },
    /// A mouse click event, fired on mouseDown
    MouseButtonDown {
        /// The mouse button being pressed
        mouse_btn: MouseButton,
        /// The location of the mouse in Simulator coordinates
        point: Point,
    },
    /// A mouse wheel event
    MouseWheel {
        /// The scroll wheel delta in the x and y direction
        scroll_delta: Point,
        /// The directionality of the scroll (normal or flipped)
        direction: MouseWheelDirection,
    },
    /// Mouse move event
    MouseMove {
        /// The current mouse position
        point: Point,
    },
    /// An exit event
    Quit,
}

/// Iterator over simulator events.
///
/// See [`Window::events`](crate::Window::events) and
/// [`MultiWindow::events`](crate::MultiWindow::events) for more details.
pub struct SimulatorEventsIter<'a> {
    event_pump: RefMut<'a, EventPump>,
    output_settings: OutputSettings,
}

impl Iterator for SimulatorEventsIter<'_> {
    type Item = SimulatorEvent;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.event_pump.poll_event() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Some(SimulatorEvent::Quit),
                Event::KeyDown {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    return keycode.map(|valid_keycode| SimulatorEvent::KeyDown {
                        keycode: valid_keycode,
                        keymod,
                        repeat,
                    })
                }
                Event::KeyUp {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    return keycode.map(|valid_keycode| SimulatorEvent::KeyUp {
                        keycode: valid_keycode,
                        keymod,
                        repeat,
                    })
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    let point = self.output_settings.output_to_display(Point::new(x, y));
                    return Some(SimulatorEvent::MouseButtonUp { point, mouse_btn });
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    let point = self.output_settings.output_to_display(Point::new(x, y));
                    return Some(SimulatorEvent::MouseButtonDown { point, mouse_btn });
                }
                Event::MouseMotion { x, y, .. } => {
                    let point = self.output_settings.output_to_display(Point::new(x, y));
                    return Some(SimulatorEvent::MouseMove { point });
                }
                Event::MouseWheel {
                    x, y, direction, ..
                } => {
                    return Some(SimulatorEvent::MouseWheel {
                        scroll_delta: Point::new(x, y),
                        direction,
                    })
                }
                _ => {
                    // ignore other events and check next event
                }
            }
        }

        None
    }
}

pub struct SdlWindow {
    canvas: Canvas<sdl2::video::Window>,
    event_pump: RefCell<EventPump>,
    window_texture: SdlWindowTexture,
    size: Size,
}

impl SdlWindow {
    pub fn new(title: &str, size: Size) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(title, size.width, size.height)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let window_texture = SdlWindowTextureBuilder {
            texture_creator: canvas.texture_creator(),
            texture_builder: |creator: &TextureCreator<WindowContext>| {
                creator
                    .create_texture_streaming(PixelFormatEnum::RGB24, size.width, size.height)
                    .unwrap()
            },
        }
        .build();

        Self {
            canvas,
            event_pump: RefCell::new(event_pump),
            window_texture,
            size,
        }
    }

    pub fn update(&mut self, framebuffer: &OutputImage<Rgb888>) {
        self.window_texture.with_mut(|fields| {
            fields
                .texture
                .update(
                    None,
                    framebuffer.data.as_ref(),
                    self.size.width as usize * 3,
                )
                .unwrap();
        });

        self.canvas
            .copy(self.window_texture.borrow_texture(), None, None)
            .unwrap();
        self.canvas.present();
    }

    /// Handle events
    /// Return an iterator of all captured SimulatorEvent
    pub fn events(&self, output_settings: &OutputSettings) -> SimulatorEventsIter<'_> {
        SimulatorEventsIter {
            event_pump: self.event_pump.borrow_mut(),
            output_settings: output_settings.clone(),
        }
    }
}

#[ouroboros::self_referencing]
struct SdlWindowTexture {
    texture_creator: TextureCreator<WindowContext>,
    #[borrows(texture_creator)]
    #[covariant]
    texture: Texture<'this>,
}
