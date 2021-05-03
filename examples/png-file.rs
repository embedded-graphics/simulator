use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};

fn main() {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(256, 64));

    let large_text = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    let centered = TextStyleBuilder::new()
        .baseline(Baseline::Middle)
        .alignment(Alignment::Center)
        .build();

    Text::with_text_style(
        "embedded-graphics",
        display.bounding_box().center(),
        large_text,
        centered,
    )
    .draw(&mut display)
    .unwrap();

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let output_image = display.to_rgb_output_image(&output_settings);

    let path = std::env::args_os()
        .nth(1)
        .expect("expected PNG file name argument");
    output_image.save_png(path).unwrap();
}
