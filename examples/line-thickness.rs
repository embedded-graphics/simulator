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
    primitives::{common::LineJoin, common::StrokeOffset, line::Intersection, Polyline},
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

    // {
    //     let l1 = Line::new(Point::new(50, 150), Point::new(150, 50));
    //     let l2 = Line::new(Point::new(50, 50), position);

    //     // let l1 = Line::new(Point::new(50, 150), Point::new(100, 100));
    //     // let l2 = Line::new(Point::new(100, 100), position);

    //     l1.into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    //         .draw(display)?;
    //     l2.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
    //         .draw(display)?;

    //     let intersection = l1.intersection(&l2);

    //     match intersection {
    //         Intersection::Point { point, outer_side } => {
    //             empty_crosshair(point, Rgb888::MAGENTA, display);
    //         }
    //         Intersection::Colinear => println!("Colinear"),
    //     }
    // }

    // ---

    // {
    //     println!("---");
    //     // 3 points almost on a straight line -> doesn't work
    //     let points = [Point::new(9, 24), Point::new(14, 14), position];
    //     // let points = [Point::new(10, 70), Point::new(20, 50), position];

    //     for width in 10..11 {
    //         let pos = Point::new(width as i32 * 30, 0);

    //         Polyline::new(&points)
    //             .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, width))
    //             .draw(&mut display.translated(pos))?;

    //         Text::new(&width.to_string(), pos + Point::new(10, 80))
    //             .into_styled(MonoTextStyle::new(Font6x8, Rgb888::WHITE))
    //             .draw(display)?;
    //     }

    //     // // 3 points on a straight line -> works
    //     // let points2 = [Point::new(10, 70), Point::new(20, 50), Point::new(30, 30)];

    //     // for width in 1..20 {
    //     //     let pos = Point::new(width as i32 * 30, 100);

    //     //     Polyline::new(&points2)
    //     //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, width))
    //     //         .draw(&mut display.translated(pos))?;

    //     //     Text::new(&width.to_string(), pos + Point::new(10, 80))
    //     //         .into_styled(MonoTextStyle::new(Font6x8, Rgb888::WHITE))
    //     //         .draw(display)?;
    //     // }
    // }

    // ---

    // {
    //     println!("---");

    //     let width = 19;

    //     // 3 points almost on a straight line -> doesn't work
    //     // let points = [Point::new(10, 70), Point::new(20, 50), Point::new(29, 30)];
    //     let points = [Point::new(10, 70), Point::new(20, 50), position];

    //     let right1 = Line::new(points[0], points[1])
    //         .extents(width, StrokeOffset::None)
    //         .1;
    //     let right2 = Line::new(points[2], points[1])
    //         .extents(width, StrokeOffset::None)
    //         .0;

    //     // right1
    //     //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
    //     //     .draw(display)?;

    //     // right2
    //     //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
    //     //     .draw(display)?;

    //     let (a, b) = Line::new_intersection(&right1, &right2);

    //     let word = if a.signum() != b.signum() {
    //         "Bad"
    //     } else {
    //         "Good"
    //     };

    //     Text::new(word, Point::new(0, 0))
    //         .into_styled(MonoTextStyle::new(Font6x8, Rgb888::WHITE))
    //         .draw(display)?;

    //     match right1.intersection(&right2) {
    //         Intersection::Point { point, outer_side } => {
    //             empty_crosshair(point, Rgb888::MAGENTA, display);
    //         }
    //         Intersection::Colinear => println!("Colinear"),
    //     }

    //     for blah in points.windows(2) {
    //         if let [p1, p2] = blah {
    //             Line::new(*p1, *p2)
    //                 .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    //                 .draw(display)?;
    //         }
    //     }
    // }

    // ---

    {
        println!("---");

        let width = 19;

        // 3 points almost on a straight line -> doesn't work
        // let points = [Point::new(10, 70), Point::new(20, 50), Point::new(29, 30)];
        let points = [Point::new(10, 70), Point::new(20, 50), position];

        let intersections = [
            LineJoin::start(points[0], points[1], width, StrokeOffset::None),
            LineJoin::from_points(points[0], points[1], points[2], width, StrokeOffset::None),
            LineJoin::end(points[1], points[2], width, StrokeOffset::None),
        ];

        for blah in intersections.windows(2) {
            if let [i1, i2] = blah {
                Line::new(i1.second_edge_start.right, i2.first_edge_end.right)
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
                    .draw(display)?;

                Line::new(i1.second_edge_start.left, i2.first_edge_end.left)
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
                    .draw(display)?;
            }
        }

        // Skeleton
        for blah in points.windows(2) {
            if let [p1, p2] = blah {
                Line::new(*p1, *p2)
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
                    .draw(display)?;
            }
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

    let mut position = Point::new(29, 30);
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
