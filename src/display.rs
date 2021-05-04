use std::convert::TryFrom;

use embedded_graphics::{
    pixelcolor::{BinaryColor, Gray8, Rgb888},
    prelude::*,
};

use crate::{output_image::OutputImage, output_settings::OutputSettings};

/// Simulator display.
pub struct SimulatorDisplay<C> {
    size: Size,
    pixels: Box<[C]>,
}

impl<C: PixelColor> SimulatorDisplay<C> {
    /// Creates a new display filled with a color.
    ///
    /// This constructor can be used if `C` doesn't implement `From<BinaryColor>` or another
    /// default color is wanted.
    pub fn with_default_color(size: Size, default_color: C) -> Self {
        let pixel_count = size.width as usize * size.height as usize;
        let pixels = vec![default_color; pixel_count].into_boxed_slice();

        SimulatorDisplay { size, pixels }
    }

    /// Returns the color of the pixel at a point.
    ///
    /// # Panics
    ///
    /// Panics if `point` is outside the display.
    pub fn get_pixel(&self, point: Point) -> C {
        self.point_to_index(point)
            .and_then(|index| self.pixels.get(index).copied())
            .expect("can't get point outside of display")
    }

    fn point_to_index(&self, point: Point) -> Option<usize> {
        if let Ok((x, y)) = <(u32, u32)>::try_from(point) {
            if x < self.size.width && y < self.size.height {
                return Some((x + y * self.size.width) as usize);
            }
        }

        None
    }
}

impl<C> SimulatorDisplay<C>
where
    C: PixelColor + From<BinaryColor>,
{
    /// Creates a new display.
    ///
    /// The display is filled with `C::from(BinaryColor::Off)`.
    pub fn new(size: Size) -> Self {
        Self::with_default_color(size, C::from(BinaryColor::Off))
    }
}

impl<C> SimulatorDisplay<C>
where
    C: PixelColor + Into<Rgb888>,
{
    /// Converts the display contents into a RGB output image.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
    /// use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
    ///
    /// let output_settings = OutputSettingsBuilder::new().scale(2).build();
    ///
    /// let display = SimulatorDisplay::<Rgb888>::new(Size::new(128, 64));
    ///
    /// // draw something to the display
    ///
    /// let output_image = display.to_rgb_output_image(&output_settings);
    /// assert_eq!(output_image.size(), Size::new(256, 128));
    ///
    /// // use output image:
    /// // example: output_image.save_png("out.png")?;
    /// ```
    pub fn to_rgb_output_image(&self, output_settings: &OutputSettings) -> OutputImage<Rgb888> {
        let mut output = OutputImage::new(self, output_settings);
        output.update(self);

        output
    }

    /// Converts the display contents into a grayscale output image.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use embedded_graphics::{pixelcolor::Gray8, prelude::*};
    /// use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
    ///
    /// let output_settings = OutputSettingsBuilder::new().scale(2).build();
    ///
    /// let display = SimulatorDisplay::<Gray8>::new(Size::new(128, 64));
    ///
    /// // draw something to the display
    ///
    /// let output_image = display.to_grayscale_output_image(&output_settings);
    /// assert_eq!(output_image.size(), Size::new(256, 128));
    ///
    /// // use output image:
    /// // example: output_image.save_png("out.png")?;
    /// ```
    pub fn to_grayscale_output_image(
        &self,
        output_settings: &OutputSettings,
    ) -> OutputImage<Gray8> {
        let mut output = OutputImage::new(self, output_settings);
        output.update(self);

        output
    }
}

impl<C: PixelColor> DrawTarget for SimulatorDisplay<C> {
    type Color = C;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            if let Some(index) = self.point_to_index(point) {
                self.pixels[index] = color;
            }
        }

        Ok(())
    }
}

impl<C> OriginDimensions for SimulatorDisplay<C> {
    fn size(&self) -> Size {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use embedded_graphics::primitives::{Line, PrimitiveStyle};

    use super::*;

    #[test]
    fn rgb_output_image() {
        let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(2, 4));

        Line::new(Point::new(0, 0), Point::new(1, 3))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        let image = display.to_rgb_output_image(&OutputSettings::default());
        assert_eq!(image.size(), display.size());

        let expected: &[u8] = &[
            255, 255, 255, 0, 0, 0, //
            255, 255, 255, 0, 0, 0, //
            0, 0, 0, 255, 255, 255, //
            0, 0, 0, 255, 255, 255, //
        ];
        assert_eq!(image.data.as_ref(), expected);
    }

    #[test]
    fn grayscale_image_buffer() {
        let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(2, 4));

        Line::new(Point::new(0, 0), Point::new(1, 3))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        let image = display.to_grayscale_output_image(&OutputSettings::default());
        assert_eq!(image.size(), display.size());

        let expected: &[u8] = &[
            255, 0, //
            255, 0, //
            0, 255, //
            0, 255, //
        ];
        assert_eq!(image.data.as_ref(), expected);
    }
}
