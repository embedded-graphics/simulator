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
    primitives::line::StrokeOffset,
    primitives::*,
    style::{PrimitiveStyle, TextStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::keyboard::Keycode;

fn draw(
    display: &mut SimulatorDisplay<Rgb888>,
    position: Point,
    stroke_width: u32,
) -> Result<(), core::convert::Infallible> {
    display.clear(Rgb888::BLACK)?;

    let start = Point::new(
        display.size().width as i32 / 2,
        display.size().height as i32 / 2,
    );

    Text::new(
        &format!(
            "W: {}\nDX {}, DY {}",
            stroke_width,
            position.x - start.x,
            position.y - start.y
        ),
        Point::zero(),
    )
    .into_styled(TextStyle::new(Font6x8, Rgb888::MAGENTA))
    .draw(display)?;

    let offset = StrokeOffset::None;
    let p1 = Point::new(80, 150);
    let p2 = Point::new(120, 80);
    let p3 = position;

    let t = Triangle::new(p1, p2, p3).sorted_clockwise();

    let joins = [
        LineJoin::from_points(t.p3, t.p1, t.p2, stroke_width, offset),
        LineJoin::from_points(t.p1, t.p2, t.p3, stroke_width, offset),
        LineJoin::from_points(t.p2, t.p3, t.p1, stroke_width, offset),
    ];

    let sides = [
        ThickSegment::new(joins[0], joins[1]),
        ThickSegment::new(joins[1], joins[2]),
        ThickSegment::new(joins[2], joins[0]),
    ];

    sides.iter().enumerate().try_for_each(|(idx, side)| {
        // Outside is always left side of line due to clockwise sorting.
        let (inside, outside) = side.edges();

        outside
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_CORAL, 1))
            .draw(display)?;

        inside
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DEEP_SKY_BLUE, 1))
            .draw(display)?;

        Text::new(&format!("P{}", idx + 1), outside.start)
            .into_styled(TextStyle::new(Font6x8, Rgb888::CSS_YELLOW_GREEN))
            .draw(display)
    })?;

    // t.into_styled(PrimitiveStyle::with_stroke(
    //     Rgb888::new(0x80, 0xf2, 0x91),
    //     stroke_width,
    // ))
    // .draw(display)?;

    Ok(())
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(200, 200));
    let output_settings = OutputSettingsBuilder::new()
        .scale(4)
        .pixel_spacing(1)
        .build();
    let mut window = Window::new("Thick triangles", &output_settings);

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
