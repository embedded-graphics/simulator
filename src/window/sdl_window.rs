use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{PixelColor, Point, Size},
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

use crate::{OutputImage, OutputSettings, SimulatorDisplay};

struct SdlContext {
    video_subsystem: sdl2::VideoSubsystem,
    event_pump: EventPump,
}

thread_local! {
    static SDL_CONTEXT: std::cell::RefCell<SdlContext> = {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        std::cell::RefCell::new(SdlContext {
            video_subsystem,
            event_pump,
        })
    };
}

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

pub struct SdlWindow {
    canvas: Canvas<sdl2::video::Window>,
    window_texture: SdlWindowTexture,
    size: Size,
}

impl SdlWindow {
    pub fn new<C>(
        display: &SimulatorDisplay<C>,
        title: &str,
        output_settings: &OutputSettings,
    ) -> Self
    where
        C: PixelColor + Into<Rgb888>,
    {
        let size = output_settings.framebuffer_size(display);

        let window = SDL_CONTEXT.with(|ctx| {
            ctx.borrow_mut()
                .video_subsystem
                .window(title, size.width, size.height)
                .position_centered()
                .build()
                .unwrap()
        });

        let canvas = window.into_canvas().build().unwrap();

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
            .copy(&self.window_texture.borrow_texture(), None, None)
            .unwrap();
        self.canvas.present();
    }

    /// Handle events
    /// Return an iterator of all captured SimulatorEvent
    pub fn events(
        &mut self,
        output_settings: &OutputSettings,
    ) -> impl Iterator<Item = SimulatorEvent> + '_ {
        let output_settings = output_settings.clone();
        let events: Vec<Event> =
            SDL_CONTEXT.with(|ctx| ctx.borrow_mut().event_pump.poll_iter().collect());
        events
            .into_iter()
            .filter(|e| e.get_window_id() == Some(self.canvas.window().id()))
            .filter_map(move |event| match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => Some(SimulatorEvent::Quit),
                Event::KeyDown {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    if let Some(valid_keycode) = keycode {
                        Some(SimulatorEvent::KeyDown {
                            keycode: valid_keycode,
                            keymod,
                            repeat,
                        })
                    } else {
                        None
                    }
                }
                Event::KeyUp {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    if let Some(valid_keycode) = keycode {
                        Some(SimulatorEvent::KeyUp {
                            keycode: valid_keycode,
                            keymod,
                            repeat,
                        })
                    } else {
                        None
                    }
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    let point = output_settings.output_to_display(Point::new(x, y));
                    Some(SimulatorEvent::MouseButtonUp { point, mouse_btn })
                }
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    let point = output_settings.output_to_display(Point::new(x, y));
                    Some(SimulatorEvent::MouseButtonDown { point, mouse_btn })
                }
                Event::MouseWheel {
                    x, y, direction, ..
                } => Some(SimulatorEvent::MouseWheel {
                    scroll_delta: Point::new(x, y),
                    direction,
                }),
                Event::MouseMotion { x, y, .. } => {
                    let point = output_settings.output_to_display(Point::new(x, y));
                    Some(SimulatorEvent::MouseMove { point })
                }
                _ => None,
            })
    }
}

#[ouroboros::self_referencing]
struct SdlWindowTexture {
    texture_creator: TextureCreator<WindowContext>,
    #[borrows(texture_creator)]
    #[covariant]
    texture: Texture<'this>,
}
