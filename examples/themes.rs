use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};

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

    // Uncomment one of the `theme` lines to use a different theme.
    let output_settings = OutputSettingsBuilder::new()
        //.theme(BinaryColorTheme::LcdGreen)
        //.theme(BinaryColorTheme::LcdWhite)
        .theme(BinaryColorTheme::LcdBlue)
        //.theme(BinaryColorTheme::OledBlue)
        //.theme(BinaryColorTheme::OledWhite)
        .build();

    let mut window = Window::new("Themes", &output_settings);
    window.show_static(&display);
}
