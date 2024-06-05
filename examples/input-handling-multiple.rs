//! # Example: Input Handling
//!
//! This example allows you to move a red circle to the location of a click on the simulator
//! screen, or move the circle using the arrow keys. Although input handling is not a part of the
//! embedded-graphics API, the simulator can be used to emulate input controls in order to
//! represent more complex UI systems such as touch screens.

extern crate embedded_graphics;
extern crate embedded_graphics_simulator;

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettings, SimulatorDisplay, SimulatorEvent, Window,
};

const BACKGROUND_COLOR: Rgb888 = Rgb888::BLACK;
const FOREGROUND_COLOR: Rgb888 = Rgb888::RED;
const KEYBOARD_DELTA: i32 = 20;

fn move_circle(
    display: &mut SimulatorDisplay<Rgb888>,
    old_center: Point,
    new_center: Point,
) -> Result<(), core::convert::Infallible> {
    // Clear old circle
    Circle::with_center(old_center, 200)
        .into_styled(PrimitiveStyle::with_fill(BACKGROUND_COLOR))
        .draw(display)?;

    // Draw circle at new location
    Circle::with_center(new_center, 200)
        .into_styled(PrimitiveStyle::with_fill(FOREGROUND_COLOR))
        .draw(display)?;

    Ok(())
}

enum LoopResult {
    Continue,
    AddNewWindow,
    Quit,
}

fn handle_loop(
    window: &mut Window,
    display: &mut SimulatorDisplay<Rgb888>,
    position: &mut Point,
) -> Result<LoopResult, core::convert::Infallible> {
    window.update(display);

    let events = window.events();
    for event in events {
        match event {
            SimulatorEvent::Quit => return Ok(LoopResult::Quit),
            SimulatorEvent::KeyDown { keycode, .. } => {
                let delta = match keycode {
                    Keycode::Left => Point::new(-KEYBOARD_DELTA, 0),
                    Keycode::Right => Point::new(KEYBOARD_DELTA, 0),
                    Keycode::Up => Point::new(0, -KEYBOARD_DELTA),
                    Keycode::Down => Point::new(0, KEYBOARD_DELTA),
                    Keycode::N => return Ok(LoopResult::AddNewWindow),
                    _ => Point::zero(),
                };
                let new_position = *position + delta;
                move_circle(display, *position, new_position)?;
                *position = new_position;
            }
            SimulatorEvent::MouseButtonUp { point, .. } => {
                move_circle(display, *position, point)?;
                *position = point;
            }
            _ => {}
        }
    }
    Ok(LoopResult::Continue)
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut windows = vec![(
        SimulatorDisplay::new(Size::new(800, 480)),
        Window::new(
            "Click to move circle (press N for new window)",
            &OutputSettings::default(),
        ),
        Point::new(200, 200),
    )];

    for (display, _, position) in windows.iter_mut() {
        Circle::with_center(*position, 200)
            .into_styled(PrimitiveStyle::with_fill(FOREGROUND_COLOR))
            .draw(display)?;
    }

    'running: loop {
        for (display, window, position) in windows.iter_mut() {
            match handle_loop(window, display, position)? {
                LoopResult::Continue => {}
                LoopResult::AddNewWindow => {
                    let mut display = SimulatorDisplay::new(Size::new(800, 480));
                    let window = Window::new(
                        "Click to move circle (press N for new window)",
                        &OutputSettings::default(),
                    );
                    let position = Point::new(200, 200);

                    Circle::with_center(position, 200)
                        .into_styled(PrimitiveStyle::with_fill(FOREGROUND_COLOR))
                        .draw(&mut display)?;

                    windows.push((display, window, position));
                    break;
                }
                LoopResult::Quit => {
                    break 'running;
                }
            }
        }
    }

    Ok(())
}
