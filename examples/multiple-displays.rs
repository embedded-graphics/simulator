//! # Example: Multiple displays
//!
//! This example demonstrates how multiple displays can be displayed in a common window.

extern crate embedded_graphics;
extern crate embedded_graphics_simulator;

use embedded_graphics::{
    geometry::AnchorPoint,
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb565, Rgb888},
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::{Alignment, Baseline, Text, TextStyle, TextStyleBuilder},
};
use embedded_graphics_simulator::{
    sdl2::MouseButton, BinaryColorTheme, MultiWindow, OutputSettings, OutputSettingsBuilder,
    SimulatorDisplay, SimulatorEvent,
};

const OLED_TEXT: MonoTextStyle<BinaryColor> = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
const TFT_TEXT: MonoTextStyle<Rgb565> =
    MonoTextStyle::new(&FONT_10X20, Rgb565::CSS_LIGHT_SLATE_GRAY);
const CENTERED: TextStyle = TextStyleBuilder::new()
    .alignment(Alignment::Center)
    .baseline(Baseline::Middle)
    .build();

/// Determines the position of a display.
fn display_offset(window_size: Size, display_size: Size, anchor_point: AnchorPoint) -> Point {
    // Position displays in a rectangle that is 20px than the the window.
    let layout_rect = Rectangle::new(Point::zero(), window_size).offset(-20);

    // Resize the rectangle to the display size to determine the offset from the
    // top left corner of the window to the top left corner of the display.
    layout_rect.resized(display_size, anchor_point).top_left
}

fn main() -> Result<(), core::convert::Infallible> {
    // Create three simulated monochrome 128x64 OLED displays.

    let mut oled_displays = Vec::new();
    for i in 0..3 {
        let mut oled: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(128, 64));

        Text::with_text_style(
            &format!("Display {i}"),
            oled.bounding_box().center(),
            OLED_TEXT,
            CENTERED,
        )
        .draw(&mut oled)
        .unwrap();

        oled_displays.push(oled);
    }

    // Create a simulated color 320x240 TFT display.

    let mut tft: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));
    tft.clear(Rgb565::new(5, 10, 5)).unwrap();

    Text::with_text_style(
        &format!("Draw here"),
        tft.bounding_box().center(),
        TFT_TEXT,
        CENTERED,
    )
    .draw(&mut tft)
    .unwrap();

    // The simulated displays can now be added to common simulator window.

    let window_size = Size::new(1300, 500);
    let mut window = MultiWindow::new("Multiple displays example", window_size);
    window.clear(Rgb888::CSS_DIM_GRAY);

    let oled_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .scale(2)
        .build();
    let oled_size = oled_displays[0].output_size(&oled_settings);

    for (oled, anchor) in oled_displays.iter().zip(
        [
            AnchorPoint::TopLeft,
            AnchorPoint::TopCenter,
            AnchorPoint::TopRight,
        ]
        .into_iter(),
    ) {
        let offset = display_offset(window_size, oled_size, anchor);
        window.add_display(&oled, offset, &oled_settings);
    }

    let tft_settings = OutputSettings::default();
    let tft_size = tft.output_size(&tft_settings);
    let tft_offset = display_offset(window_size, tft_size, AnchorPoint::BottomCenter);

    window.add_display(&tft, tft_offset, &tft_settings);

    let border_style = PrimitiveStyleBuilder::new()
        .stroke_width(5)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();

    let mut mouse_down = false;

    'running: loop {
        // Call `update_display` for all display. Note that the window won't be
        // updated until `window.flush` is called.
        for oled in &oled_displays {
            window.update_display(oled);
        }
        window.update_display(&tft);
        window.flush();

        for event in window.events() {
            match event {
                SimulatorEvent::MouseMove { point } => {
                    // Mouse events use the window coordinate system.
                    // `translate_mouse_position` can be used to translate the
                    // mouse position into the display coordinate system.

                    for oled in &mut oled_displays {
                        let is_inside = window.translate_mouse_position(oled, point).is_some();

                        let style = PrimitiveStyleBuilder::from(&border_style)
                            .stroke_color(BinaryColor::from(is_inside))
                            .build();

                        oled.bounding_box().into_styled(style).draw(oled).unwrap();
                    }

                    if mouse_down {
                        if let Some(point) = window.translate_mouse_position(&tft, point) {
                            Circle::with_center(point, 10)
                                .into_styled(PrimitiveStyle::with_fill(Rgb565::CSS_DODGER_BLUE))
                                .draw(&mut tft)
                                .unwrap();
                        }
                    }
                }
                SimulatorEvent::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    mouse_down = true;
                }
                SimulatorEvent::MouseButtonUp {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    mouse_down = false;
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}
