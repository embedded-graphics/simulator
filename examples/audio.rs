//! # Example: Audio
//!
//! This example allows you to

extern crate embedded_graphics;
extern crate embedded_graphics_simulator;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::audio::{AudioCallback, AudioSpecDesired};

const SAMPLE_RATE: i32 = 44100;

const PITCH_MIN: f32 = 440.0;
const PITCH_MAX: f32 = 10000.0;

fn main() -> Result<(), core::convert::Infallible> {
    let gate = Arc::new(AtomicBool::new(false));
    let audio_wrapper = AudioWrapper::new(gate.clone());

    let audio_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(1),
        samples: Some(32),
    };
    let audio_device = sdl2::init()
        .unwrap()
        .audio()
        .unwrap()
        .open_playback(None, &audio_spec, |_| audio_wrapper)
        .unwrap();

    audio_device.resume();

    let output_settings = OutputSettingsBuilder::new()
        .scale(4)
        .theme(embedded_graphics_simulator::BinaryColorTheme::OledWhite)
        .build();
    let mut window = Window::new("Simulator audio example", &output_settings);

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let text_position = Point::new(25, 30);
    let text = Text::new("Press space...", text_position, text_style);

    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(128, 64));
    _ = text.draw(&mut display);
    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Space => {
                            gate.store(true, Ordering::SeqCst);
                            _ = display.clear(BinaryColor::On);
                        }
                        _ => {}
                    };
                }
                SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                    Keycode::Space => {
                        gate.store(false, Ordering::SeqCst);
                        _ = display.clear(BinaryColor::Off);
                        _ = text.draw(&mut display);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok(())
}

struct AudioWrapper {
    gate: Arc<AtomicBool>,
    phase: f32,
    pitch: f32,
}

impl AudioWrapper {
    fn new(gate: Arc<AtomicBool>) -> Self {
        Self {
            gate,
            phase: 0.0,
            pitch: PITCH_MIN,
        }
    }
}

impl AudioCallback for AudioWrapper {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let gate = self.gate.load(Ordering::SeqCst);
        for x in out.iter_mut() {
            if gate {
                self.phase += self.pitch / SAMPLE_RATE as f32;
                *x = self.phase.sin();

                if self.pitch > PITCH_MAX {
                    self.pitch = PITCH_MIN;
                }

                self.pitch += 0.5;
            } else {
                *x = 0.0
            }
        }
    }
}
