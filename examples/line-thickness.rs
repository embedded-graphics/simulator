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
    primitives::{common::StrokeOffset, line::Intersection, Polyline},
    style::{MonoTextStyle, PrimitiveStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::keyboard::Keycode;

const BACKGROUND_COLOR: Rgb888 = Rgb888::BLACK;

fn empty_crosshair<D>(
    point: Point,
    color: Rgb888,
    display: &mut D,
) -> Result<(), core::convert::Infallible>
where
    D: DrawTarget<Color = Rgb888, Error = core::convert::Infallible>,
{
    let radius = Size::new_equal(4);
    let inner_radius = Size::new_equal(2);

    Line::new(point - radius.x_axis(), point - inner_radius.x_axis())
        .points()
        .chain(Line::new(point + radius.x_axis(), point + inner_radius.x_axis()).points())
        .chain(Line::new(point - radius.y_axis(), point - inner_radius.y_axis()).points())
        .chain(Line::new(point + radius.y_axis(), point + inner_radius.y_axis()).points())
        .map(|p| Pixel(p, color))
        .draw(display)
}

fn draw(
    display: &mut SimulatorDisplay<Rgb888>,
    position: Point,
    stroke_width: u32,
) -> Result<(), core::convert::Infallible> {
    display.clear(BACKGROUND_COLOR)?;

    let mut display = display.translated(Point::new(20, 20));

    let width = 19;

    // 3 points almost on a straight line -> doesn't work
    let points = [Point::new(10, 70), Point::new(20, 50), Point::new(29, 30)];

    let right1 = Line::new(points[0], points[1])
        .extents(width, StrokeOffset::None)
        .1;
    let right2 = Line::new(points[1], points[2])
        .extents(width, StrokeOffset::None)
        .1;

    right1
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
        .draw(&mut display)?;

    right2
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        .draw(&mut display)?;

    let intersection = right1.intersection(&right2);

    match intersection {
        Intersection::Point { point, outer_side } => {
            empty_crosshair(point, Rgb888::MAGENTA, &mut display);
        }
        Intersection::Colinear => println!("Colinear"),
    }

    for blah in points.windows(2) {
        if let [p1, p2] = blah {
            Line::new(*p1, *p2)
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
                .draw(&mut display)?;
        }
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
