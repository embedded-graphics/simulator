//! A debugging tool for thick lines
//!
//! Use the up/down arrow keys to increase or decrease the line thickness. Click and drag to move
//! the end point of the line around.
//!
//! The thickness, DX and DY components of the line are displayed in the top right corner of the
//! window.

extern crate embedded_graphics;
extern crate embedded_graphics_simulator;

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::Line,
    primitives::Polyline,
    style::{MonoTextStyle, PrimitiveStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::keyboard::Keycode;

const BACKGROUND_COLOR: Rgb888 = Rgb888::BLACK;

fn draw(
    display: &mut SimulatorDisplay<Rgb888>,
    position: Point,
    stroke_width: u32,
) -> Result<(), core::convert::Infallible> {
    display.clear(BACKGROUND_COLOR)?;

    // 3 points almost on a straight line -> doesn't work
    let points = [Point::new(10, 70), Point::new(20, 50), Point::new(29, 30)];

    for width in 1..20 {
        let pos = Point::new(width as i32 * 30, 0);

        Polyline::new(&points)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, width))
            .draw(&mut display.translated(pos))?;

        Text::new(&width.to_string(), pos + Point::new(10, 80))
            .into_styled(MonoTextStyle::new(Font6x8, Rgb888::WHITE))
            .draw(display)?;
    }

    // 3 points on a straight line -> works
    let points2 = [Point::new(10, 70), Point::new(20, 50), Point::new(30, 30)];

    for width in 1..20 {
        let pos = Point::new(width as i32 * 30, 100);

        Polyline::new(&points2)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, width))
            .draw(&mut display.translated(pos))?;

        Text::new(&width.to_string(), pos + Point::new(10, 80))
            .into_styled(MonoTextStyle::new(Font6x8, Rgb888::WHITE))
            .draw(display)?;
    }

    Ok(())
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(700, 200));
    let output_settings = OutputSettingsBuilder::new()
        .scale(2)
        // .pixel_spacing(1)
        .build();
    let mut window = Window::new("Line thickness debugger", &output_settings);

    let mut position = Point::new(150, 120);
    let mut stroke_width = 5;
    let mut mouse_down = false;

    draw(&mut display, position, stroke_width)?;

    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Up => stroke_width += 1,
                        Keycode::Down => stroke_width = (stroke_width as i32 - 1).max(0) as u32,
                        _ => (),
                    }

                    draw(&mut display, position, stroke_width)?;
                }
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    mouse_down = true;
                    position = point;

                    draw(&mut display, position, stroke_width)?;
                }
                SimulatorEvent::MouseButtonUp { .. } => mouse_down = false,
                SimulatorEvent::MouseMove { point, .. } => {
                    if mouse_down {
                        position = point;
                        draw(&mut display, position, stroke_width)?;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
