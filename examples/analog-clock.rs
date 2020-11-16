//! # Example: Analog Clock
//!
//! ![Screenshot of clock example]( data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAfwAAAH9AQMAAADFwFz1AAAABlBMVEUAAAD///+l2Z/dAAAFM0lEQVR42u3dYWrjMBAF4Afz1zBX8QEEc/WBvdAcQKBNussK3EbVpDNSt+SRYMWWPrBsybQJCK+88sp/EWm32BagN+/EcoBuLRUE8K2wA6B7c9yBN2I9QK3+2eItrS0HWv0L9c9LgV6f+p6lADdcAWq6AuiVO9DRhYBYty571wBc8RGApquApkM4H+A6lvOBpkM6H+A6tvOBpkM8H6A61BcAosMetnSA6olBzpYOsI0B0Wyg4cAgSjUZIPsEgCQDoqCRD7DlAvX2Hlyo414lFSD7HJAsoI/lc3An9xGdBFRMpCYCZDOAJAKsMwBrHiCYCVka0I+MI0GAvw97vSxAMBeyLKBiMjUJIJsFJAjwX8VeMwcQzIYsB6iYTk0ByOYBCQL8V7HXzQAE8yHLAAocSQHMA5QEgNQDcALAPkDjAYEnZPFAhSs1HjAfUMIBUh/A4QB7AY0GBL6QRQMFzoQD5gVKMEBuQKIB9QIcDDC8IY0FxA9YLFDgTjBQ/UCNBcwPlFCA1A9wKMBPAE1DgeI/AwoFxN+LJ1kkUHD655NQoLrntAOokYAB6h9NgQAZnohEAvoMwFHA+9HcHuUCaCCAZwAKBOQ5wOKAcgGU7q/WLoULgB8FXMZSlVqkisq1cAHKDwJIrzdSlXZvdysU6YUrwD8ZqHR/3VKpKeRvgVoawHgOIE0DfrX7S6X35q/bpkoaIM8CFgWUy4G3VnegSr21fSt8BOAnA2+75O9cet9WQG7lNMCeBUoW0B7kBwOkzwIcB/iSD+h2oKwGnv8XMWkSgHM7wNsB2Fqg4OlzsDQAuh0o2wHS3QDO7QCvBGxw7JOUTODYDpDuBlC2A6S7AZzbAd4OwBYB/XL5xxMnA6S7AZTtAG0HYNsB3g7AWDcDR6ubAfoGQFsAtGbPAdLal4Fv0gcjAK1+EfgGN9IXgcK6GVBsBhi7gXM7oDdjK8DYDZxrANioD8dASQYYO4F/bY+NAOlu4MBu4NwNkG4FekvKBUZ/WtkEYJkAYTdwbAesz6x7AMJu4FgJ8INDnwOkeQBhN1B68dwD6FKAdNCHQ4CRBpTtgO4G+DKws4H3T9dzEihpgO4GGKuBMujDIWBZgF6G9nKAsRyQQR+OAAoD+DJXTwOaAxxYCPTCaBLVwYyWAZDuBg5sAGDosXmgpACELUAv4XB8fWwpgO0B5NKHcwAFAtz70ANoIPCvaA6AA4E+O3q+QWfEAf2CeAAJBPAUUCKBCqgPOIAaCRQwfAApLBIQnHD+iOCkUIBJvQCThgIVXgAtFCD1A4xIYDyaPtZLLFD9QB0CbqD4AYsFxA1QMMB+QGOB/nH26coIBswLSDAAN1CigeIFLBoQJ0DhAKvv6coaDZAXQDQA8wElHqg+oMYD4gLI4gF2/daTNR4gH4B4AOYBSgZQPIBlAOIAKALw3En6vm4GQDYPCDIA1Hmg5gAyDZDlAKyzAGsOQDb7fBfkAKizQM0CZBIgywJY5wDWLIBsDhBkAZCp5ztZHsD6ALZxLT/g7UU9LicaALiGE4EulfIAUQD6wVXUfj3ZMgGy2xuPAep9mASgfg7UXEAUxwCAgi0XIMM5Ak4IcgE0nKNb+aSaDbDpaDgfotkAVR1NaUdDBOBfWkj7GeYDVEdAQz6Apo+frlxTgUGls+OpwONaZ7dXAFzfA51eAfR6HehyCOBfdu/o8BIAYleg710DUNP3ADesAnrlDnR0EYBWL0Dfkw90oQPU24cA/uVYqTV8Ffj/1rT9Buv6foO1jb/D+s6vvPKKI78B89G0YRkxkl8AAAAASUVORK5CYII=)
//!
//! This example shows some more advanced usage of Embedded Graphics. It draws a round clock face
//! with hour, minute and second hands. A digital clock is drawn in the middle of the clock. The
//! whole thing is updated with your computer's local time every 50ms.

use chrono::{Local, Timelike};
use core::f32::consts::PI;
use embedded_graphics::{
    fonts::{Font12x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, Rectangle},
    style::{MonoTextStyle, PrimitiveStyle, PrimitiveStyleBuilder},
};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use std::{thread, time::Duration};

/// The margin between the clock face and the display border.
const MARGIN: u32 = 10;

/// Converts a polar coordinate (angle/distance) into an (X, Y) coordinate centered around the
/// center of the circle.
///
/// The angle is relative to the 12 o'clock position and the radius is relative to the edge of the
/// clock face.
fn polar(circle: &Circle, angle: f32, radius_delta: i32) -> Point {
    let radius = circle.diameter as f32 / 2.0 + radius_delta as f32;

    circle.center()
        + Point::new(
            (angle.sin() * radius) as i32,
            -(angle.cos() * radius) as i32,
        )
}

/// Converts an hour into an angle in radians.
fn hour_to_angle(hour: u32) -> f32 {
    // Convert from 24 to 12 hour time.
    let hour = hour % 12;

    (hour as f32 / 12.0) * 2.0 * PI
}

/// Converts a sexagesimal (base 60) value into an angle in radians.
fn sexagesimal_to_angle(value: u32) -> f32 {
    (value as f32 / 60.0) * 2.0 * PI
}

/// Creates a centered circle for the clock face.
fn create_face(target: &impl DrawTarget) -> Circle {
    // The draw target bounding box can be used to determine the size of the display.
    let bounding_box = target.bounding_box();

    let diameter = bounding_box.size.width.min(bounding_box.size.height) - 2 * MARGIN;

    Circle::with_center(bounding_box.center(), diameter)
}

/// Draws a circle and 12 graduations as a simple clock face.
fn draw_face<D>(target: &mut D, clock_face: &Circle) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // Draw the outer face.
    (*clock_face)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
        .draw(target)?;

    // Draw 12 graduations.
    for angle in (0..12).map(hour_to_angle) {
        // Start point on circumference.
        let start = polar(clock_face, angle, 0);

        // End point offset by 10 pixels from the edge.
        let end = polar(clock_face, angle, -10);

        Line::new(start, end)
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(target)?;
    }

    Ok(())
}

/// Draws a clock hand.
fn draw_hand<D>(
    target: &mut D,
    clock_face: &Circle,
    angle: f32,
    length_delta: i32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let end = polar(clock_face, angle, length_delta);

    Line::new(clock_face.center(), end)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(target)
}

/// Draws a decorative circle on the second hand.
fn draw_second_decoration<D>(
    target: &mut D,
    clock_face: &Circle,
    angle: f32,
    length_delta: i32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let decoration_position = polar(clock_face, angle, length_delta);

    let decoration_style = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::Off)
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

    // Draw a fancy circle near the end of the second hand.
    Circle::with_center(decoration_position, 11)
        .into_styled(decoration_style)
        .draw(target)
}

/// Draw digital clock just above center with black text on a white background
fn draw_digital_clock<D>(
    target: &mut D,
    clock_face: &Circle,
    time_str: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    // Create a styled text object for the time text.
    let mut text = Text::new(&time_str, Point::zero())
        .into_styled(MonoTextStyle::new(Font12x16, BinaryColor::Off));

    // Move text to be centered between the 12 o'clock point and the center of the clock face.
    text.translate_mut(
        clock_face.center()
            - text.bounding_box().center()
            - clock_face.bounding_box().size.y_axis() / 4,
    );

    // Add a background around the time digits.
    // Note that there is no bottom-right padding as this is added by the font renderer itself.
    let text_dimensions = text.bounding_box();
    Rectangle::new(
        text_dimensions.top_left - Point::new(3, 3),
        text_dimensions.size + Size::new(4, 4),
    )
    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
    .draw(target)?;

    // Draw the text after the background is drawn.
    text.draw(target)
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(256, 256));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Clock", &output_settings);

    let clock_face = create_face(&display);

    'running: loop {
        let time = Local::now();

        // Calculate the position of the three clock hands in radians.
        let hours_radians = hour_to_angle(time.hour());
        let minutes_radians = sexagesimal_to_angle(time.minute());
        let seconds_radians = sexagesimal_to_angle(time.second());

        // NOTE: In no-std environments, consider using
        // [arrayvec](https://stackoverflow.com/a/39491059/383609) and a fixed size buffer
        let digital_clock_text = format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );

        display.clear(BinaryColor::Off)?;

        draw_face(&mut display, &clock_face)?;
        draw_hand(&mut display, &clock_face, hours_radians, -60)?;
        draw_hand(&mut display, &clock_face, minutes_radians, -30)?;
        draw_hand(&mut display, &clock_face, seconds_radians, 0)?;
        draw_second_decoration(&mut display, &clock_face, seconds_radians, -20)?;

        // Draw digital clock just above center.
        draw_digital_clock(&mut display, &clock_face, &digital_clock_text)?;

        // Draw a small circle over the hands in the center of the clock face.
        // This has to happen after the hands are drawn so they're covered up.
        Circle::with_center(clock_face.center(), 9)
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display)?;

        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break 'running Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
}
