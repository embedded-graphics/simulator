//! # Example: Audio
//!
//! This example demonstrates how SDL can be used not only to implement virtual displays, but at the same time
//! to use it as an audio device. Here we implement an oscillator with a modulation of its pitch.

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

const PERIOD: f32 = 0.5; // seconds
const SAMPLES_PER_PERIOD: f32 = SAMPLE_RATE as f32 * PERIOD;
const PITCH_CHANGE_PER_SAMPLE: f32 = (PITCH_MAX - PITCH_MIN) / SAMPLES_PER_PERIOD;

fn main() -> Result<(), core::convert::Infallible> {
    // Prepare the audio "engine" with gate control
    let gate = Arc::new(AtomicBool::new(false));
    let audio_wrapper = AudioWrapper::new(gate.clone());

    let audio_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(1),
        samples: Some(32),
    };

    // This is the initialisation of SDL and capturing its audio device.
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

    // This is the window initialisation. It calls the same SDL init function inside, which
    // is allowed as long as the init calls are from the same OS thread.
    let mut window = Window::new("Simulator audio example", &output_settings);

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let text_position = Point::new(25, 30);
    let text = Text::new("Press space...", text_position, text_style);

    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(128, 64));
    text.draw(&mut display).unwrap();
    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown {
                    keycode, repeat, ..
                } if keycode == Keycode::Space && !repeat => {
                    gate.store(true, Ordering::SeqCst);
                    _ = display.clear(BinaryColor::On);
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
        if !gate {
            out.fill(0.0);
            return;
        }

        for x in out.iter_mut() {
            self.phase += self.pitch / SAMPLE_RATE as f32;
            *x = self.phase.sin();

            if self.pitch > PITCH_MAX {
                self.pitch = PITCH_MIN;
            }

            self.pitch += PITCH_CHANGE_PER_SAMPLE;
        }
    }
}
