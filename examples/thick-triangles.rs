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
    style::StrokeAlignment,
    style::{PrimitiveStyle, TextStyle},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use sdl2::keyboard::Keycode;

/// Calculate squared distance from midpoint of an outside (left) edge to the center of the triangle
fn calc_dist(center: Point, start: LineJoin, end: LineJoin) -> u32 {
    let start = start.second_edge_start.left;
    let end = end.first_edge_end.left;

    Line::new(start, end).distance_to_point_squared(center)
}

fn draw(
    display: &mut SimulatorDisplay<Rgb888>,
    mouse_pos: Point,
    position: Point,
    stroke_width: u32,
    stroke_alignment: StrokeAlignment,
) -> Result<(), core::convert::Infallible> {
    display.clear(Rgb888::BLACK)?;

    // {
    //     let l1 = Line::new(Point::new(150, 50), Point::new(100, 100));
    //     let l2 = Line::new(Point::new(100, 100), position);

    //     l1.into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    //         .draw(display)?;
    //     l2.into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
    //         .draw(display)?;

    //     let (e_l1_l, e_l1_r) = l1.extents(stroke_width, stroke_alignment.as_offset());
    //     let (e_l2_l, e_l2_r) = l2.extents(stroke_width, stroke_alignment.as_offset());

    //     let first_segment_start_edge = Line::new(e_l1_l.start, e_l1_r.start);
    //     let second_segment_end_edge = Line::new(e_l2_l.end, e_l2_r.end);

    //     e_l1_l
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    //         .draw(display)?;
    //     e_l1_r
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 1))
    //         .draw(display)?;
    //     first_segment_start_edge
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_ORANGE, 1))
    //         .draw(display)?;

    //     e_l2_l
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
    //         .draw(display)?;
    //     e_l2_r
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
    //         .draw(display)?;
    //     second_segment_end_edge
    //         .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_GREEN, 1))
    //         .draw(display)?;

    //     dbg!(
    //         first_segment_start_edge.segment_intersection(&e_l2_l),
    //         second_segment_end_edge.segment_intersection(&e_l1_r)
    //     );
    //     // dbg!(l1.line_intersection(&l2));
    // }

    let scanline = Line::new(
        Point::new(0, mouse_pos.y),
        Point::new(display.size().width as i32, mouse_pos.y),
    );
    let scanline_y = scanline.start.y;

    Text::new(&format!("{:?}", stroke_alignment), Point::zero())
        .into_styled(TextStyle::new(Font6x8, Rgb888::GREEN))
        .draw(display)?;

    scanline
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLUE, 1))
        .draw(display)?;

    let p1 = Point::new(80, 150);
    let p2 = Point::new(120, 80);
    let p3 = position;

    let points = [p1, p2, p3];

    let t = Triangle::new(p1, p2, p3).sorted_clockwise();

    let joins = [
        LineJoin::from_points(t.p3, t.p1, t.p2, stroke_width, stroke_alignment.as_offset()),
        LineJoin::from_points(t.p1, t.p2, t.p3, stroke_width, stroke_alignment.as_offset()),
        LineJoin::from_points(t.p2, t.p3, t.p1, stroke_width, stroke_alignment.as_offset()),
    ];

    let is_collapsed = joins.iter().any(|j| j.is_degenerate());

    Text::new(&format!("{:?}", is_collapsed), Point::new(0, 8))
        .into_styled(TextStyle::new(Font6x8, Rgb888::GREEN))
        .draw(display)?;

    let points = [t.p1, t.p2, t.p3];

    let it = ClosedThickSegmentIter::new(&points, stroke_width, stroke_alignment.as_offset());

    println!("---");

    // let inner_t = Triangle::new(
    //     joins[0].first_edge_end.right,
    //     joins[1].first_edge_end.right,
    //     joins[2].first_edge_end.right,
    // );

    // inner_t
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::YELLOW, 1))
    //     .draw(display)?;

    // Text::new(&format!("{:?}", inner_t.area_doubled()), Point::new(0, 16))
    //     .into_styled(TextStyle::new(Font6x8, Rgb888::GREEN))
    //     .draw(display)?;

    // let centroid = t.centroid();

    // let dist1 = calc_dist(centroid, joins[0], joins[1]);
    // let dist2 = calc_dist(centroid, joins[1], joins[2]);
    // let dist3 = calc_dist(centroid, joins[2], joins[0]);

    // // Flag denoting whether the inside of the triangle is completely filled by the edge strokes
    // // or not.
    // let is_collapsed =
    //     dist1 < stroke_width.pow(2) || dist2 < stroke_width.pow(2) || dist3 < stroke_width.pow(2);

    it.clone().enumerate().try_for_each(|(idx, side)| {
        // Outside is always left side of line due to clockwise sorting.
        let (inside, outside) = side.edges();

        // Draw bevel filler if required
        if let Some(filler) = side.start_join.filler_line() {
            filler
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DEEP_PINK, 1))
                .draw(display)?;
        }

        outside
            .into_styled(PrimitiveStyle::with_stroke(
                if !is_collapsed {
                    Rgb888::CSS_CORAL
                } else {
                    Rgb888::CSS_DARK_ORANGE
                },
                1,
            ))
            .draw(display)?;

        if !is_collapsed {
            inside
                .into_styled(PrimitiveStyle::with_stroke(Rgb888::CSS_DEEP_SKY_BLUE, 1))
                .draw(display)?;
        }

        Text::new(&format!("P{}", idx + 1), points[idx])
            .into_styled(TextStyle::new(Font6x8, Rgb888::CSS_YELLOW_GREEN))
            .draw(display)
    })?;

    // Scanline intersections
    it.filter_map(|segment| segment.intersection(scanline_y))
        .try_for_each(|line| {
            line.into_styled(PrimitiveStyle::with_stroke(Rgb888::MAGENTA, 1))
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
    let mut alignment = StrokeAlignment::Center;

    draw(&mut display, position, position, stroke_width, alignment)?;

    'running: loop {
        window.update(&display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Up => stroke_width += 1,
                        Keycode::Down => stroke_width = stroke_width.saturating_sub(1),
                        Keycode::Space => {
                            alignment = match alignment {
                                StrokeAlignment::Inside => StrokeAlignment::Center,
                                StrokeAlignment::Center => StrokeAlignment::Outside,
                                StrokeAlignment::Outside => StrokeAlignment::Inside,
                            }
                        }
                        _ => (),
                    }

                    draw(&mut display, position, position, stroke_width, alignment)?;
                }
                SimulatorEvent::MouseButtonDown { point, .. } => {
                    mouse_down = true;
                    position = point;

                    draw(&mut display, point, position, stroke_width, alignment)?;
                }
                SimulatorEvent::MouseButtonUp { .. } => mouse_down = false,
                SimulatorEvent::MouseMove { point, .. } => {
                    if mouse_down {
                        position = point;
                    }

                    draw(&mut display, point, position, stroke_width, alignment)?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
