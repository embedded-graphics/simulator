//! # Example: Primitive stroke styles
//!
//! This example demonstrates the different stroke styles available for primitives.

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, CornerRadii, Ellipse, Line, Rectangle, RoundedRectangle, Triangle},
    style::PrimitiveStyle,
};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay, Window};

const PADDING: i32 = 16;

/// Draws all embedded-graphics primitives.
fn draw_primitives<D>(target: &mut D, style: PrimitiveStyle<BinaryColor>) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    Triangle::new(Point::new(0, 64), Point::new(64, 0), Point::new(64, 64))
        .into_styled(style)
        .draw(target)?;

    Rectangle::new(Point::new(0, 0), Size::new(64, 64))
        .translate(Point::new(64 + PADDING, 0))
        .into_styled(style)
        .draw(target)?;

    Line::new(Point::new(0, 0), Point::new(64, 64))
        .translate(Point::new((64 + PADDING) * 2, 0))
        .into_styled(style)
        .draw(target)?;

    Circle::new(Point::new(0, 0), 64)
        .translate(Point::new((64 + PADDING) * 3, 0))
        .into_styled(style)
        .draw(target)?;

    RoundedRectangle::new(
        Rectangle::new(Point::new(0, 0), Size::new(64, 64)),
        CornerRadii::new(Size::new(16, 16)),
    )
    .translate(Point::new((64 + PADDING) * 4, 0))
    .into_styled(style)
    .draw(target)?;

    Ellipse::new(Point::new(0, 0), Size::new(96, 64))
        .translate(Point::new((64 + PADDING) * 5, 0))
        .into_styled(style)
        .draw(target)
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(512, 256));

    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let medium_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);
    let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 10);

    // Draw the primitives using a thin stroke.
    //
    // Instead of directly drawing to the display a `TranslatedDrawTarget` is created by
    // using `display.translated(position)`. This translates all drawing operations in `draw_shapes`
    // by 10 pixels in the x and y direction.
    let mut position = Point::new(10, 10);
    draw_primitives(&mut display.translated(position), thin_stroke)?;

    // Draw the primitives using a medium stroke.
    position.y += 64 + PADDING;
    draw_primitives(&mut display.translated(position), medium_stroke)?;

    // Draw the primitives using a thick stroke.
    position.y += 64 + PADDING;
    draw_primitives(&mut display.translated(position), thick_stroke)?;

    Window::new("Strokes", &OutputSettings::default()).show_static(&display);

    Ok(())
}
