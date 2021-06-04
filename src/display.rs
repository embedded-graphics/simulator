use std::{convert::TryFrom, fs::File, io::BufReader, path::Path};

use embedded_graphics::{
    pixelcolor::{raw::ToBytes, BinaryColor, Gray8, Rgb888},
    prelude::*,
};

use crate::{output_image::OutputImage, output_settings::OutputSettings};

/// Simulator display.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulatorDisplay<C> {
    size: Size,
    pub(crate) pixels: Box<[C]>,
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

    /// Compares the content of this display with another display.
    ///
    /// If both displays are equal `None` is returned, otherwise a difference image is returned.
    /// All pixels that are different will be filled with `BinaryColor::On` and all equal pixels
    /// with `BinaryColor::Off`.
    ///
    /// # Panics
    ///
    /// Panics if the both display don't have the same size.
    pub fn diff(&self, other: &SimulatorDisplay<C>) -> Option<SimulatorDisplay<BinaryColor>> {
        assert!(
            self.size == other.size,
            // TODO: use Display impl for Size
            "both displays must have the same size (self: {}x{}, other: {}x{})",
            self.size.width,
            self.size.height,
            other.size.width,
            other.size.height,
        );

        let pixels = self
            .bounding_box()
            .points()
            .map(|p| BinaryColor::from(self.get_pixel(p) != other.get_pixel(p)))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        if pixels.iter().any(|p| *p == BinaryColor::On) {
            Some(SimulatorDisplay {
                pixels,
                size: self.size,
            })
        } else {
            None
        }
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

impl<C> SimulatorDisplay<C>
where
    C: PixelColor + ToBytes,
    <C as ToBytes>::Bytes: AsRef<[u8]>,
{
    /// Converts the display content to big endian raw data.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_be_bytes)
    }

    /// Converts the display content to little endian raw data.
    pub fn to_le_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_le_bytes)
    }

    /// Converts the display content to native endian raw data.
    pub fn to_ne_bytes(&self) -> Vec<u8> {
        self.to_bytes(ToBytes::to_ne_bytes)
    }

    fn to_bytes<F>(&self, pixel_to_bytes: F) -> Vec<u8>
    where
        F: Fn(C) -> C::Bytes,
    {
        let mut bytes = Vec::new();

        if C::Raw::BITS_PER_PIXEL >= 8 {
            for pixel in self.pixels.iter() {
                bytes.extend_from_slice(&pixel_to_bytes(*pixel).as_ref())
            }
        } else {
            let pixels_per_byte = 8 / C::Raw::BITS_PER_PIXEL;

            for row in self.pixels.chunks(self.size.width as usize) {
                for byte_pixels in row.chunks(pixels_per_byte) {
                    let mut value = 0;

                    for pixel in byte_pixels {
                        value <<= C::Raw::BITS_PER_PIXEL;
                        value |= pixel.to_be_bytes().as_ref()[0];
                    }

                    value <<= C::Raw::BITS_PER_PIXEL * (pixels_per_byte - byte_pixels.len());

                    bytes.push(value);
                }
            }
        }

        bytes
    }
}

impl<C> SimulatorDisplay<C>
where
    C: PixelColor + From<Rgb888>,
{
    /// Loads a PNG file.
    pub fn load_png<P: AsRef<Path>>(path: P) -> image::ImageResult<Self> {
        let png_file = BufReader::new(File::open(path)?);
        let image = image::load(png_file, image::ImageFormat::Png)?.to_rgb8();

        let pixels = image
            .pixels()
            .map(|p| Rgb888::new(p[0], p[1], p[2]).into())
            .collect();

        Ok(Self {
            size: Size::new(image.width(), image.height()),
            pixels,
        })
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
    use super::*;

    use embedded_graphics::{
        pixelcolor::{Gray2, Gray4, Rgb565},
        primitives::{Circle, Line, PrimitiveStyle},
    };

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

    #[test]
    fn to_bytes_u1() {
        let display = SimulatorDisplay {
            size: Size::new(9, 3),
            pixels: [
                1, 0, 0, 0, 0, 0, 0, 1, 0, //
                0, 1, 0, 0, 0, 0, 1, 0, 1, //
                0, 0, 1, 0, 0, 1, 0, 0, 0, //
            ]
            .iter()
            .map(|c| BinaryColor::from(*c != 0))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        };

        let expected = [
            0b10000001, 0b00000000, //
            0b01000010, 0b10000000, //
            0b00100100, 0b00000000, //
        ];
        assert_eq!(&display.to_be_bytes(), &expected);
        assert_eq!(&display.to_le_bytes(), &expected);
        assert_eq!(&display.to_ne_bytes(), &expected);
    }

    #[test]
    fn to_bytes_u2() {
        let display = SimulatorDisplay {
            size: Size::new(5, 2),
            pixels: [
                0, 1, 2, 3, 0, //
                1, 0, 3, 2, 1, //
            ]
            .iter()
            .map(|c| Gray2::new(*c))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        };

        let expected = [
            0b00011011, 0b00000000, //
            0b01001110, 0b01000000, //
        ];
        assert_eq!(&display.to_be_bytes(), &expected);
        assert_eq!(&display.to_le_bytes(), &expected);
        assert_eq!(&display.to_ne_bytes(), &expected);
    }

    #[test]
    fn to_bytes_u4() {
        let display = SimulatorDisplay {
            size: Size::new(5, 4),
            pixels: [
                0x0, 0x1, 0x2, 0x3, 0x4, //
                0x5, 0x6, 0x7, 0x8, 0x9, //
                0xA, 0xB, 0xC, 0xD, 0xE, //
                0xF, 0x0, 0x0, 0x0, 0x0, //
            ]
            .iter()
            .map(|c| Gray4::new(*c))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        };

        let expected = [
            0x01, 0x23, 0x40, //
            0x56, 0x78, 0x90, //
            0xAB, 0xCD, 0xE0, //
            0xF0, 0x00, 0x00, //
        ];
        assert_eq!(&display.to_be_bytes(), &expected);
        assert_eq!(&display.to_le_bytes(), &expected);
        assert_eq!(&display.to_ne_bytes(), &expected);
    }

    #[test]
    fn to_bytes_u8() {
        let expected = [
            1, 2, 3, //
            11, 12, 13, //
        ];

        let display = SimulatorDisplay {
            size: Size::new(3, 2),
            pixels: expected
                .iter()
                .copied()
                .map(Gray8::new)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        };

        assert_eq!(&display.to_be_bytes(), &expected);
        assert_eq!(&display.to_le_bytes(), &expected);
        assert_eq!(&display.to_ne_bytes(), &expected);
    }

    #[test]
    fn to_bytes_u16() {
        let expected = vec![Rgb565::new(0x10, 0x00, 0x00), Rgb565::new(0x00, 0x00, 0x01)];

        let display = SimulatorDisplay {
            size: Size::new(2, 1),
            pixels: expected.clone().into_boxed_slice(),
        };

        assert_eq!(&display.to_be_bytes(), &[0x80, 0x00, 0x00, 0x01]);
        assert_eq!(&display.to_le_bytes(), &[0x00, 0x80, 0x01, 0x00]);
    }

    #[test]
    fn to_bytes_u24() {
        let expected = vec![Rgb888::new(0x80, 0x00, 0x00), Rgb888::new(0x00, 0x00, 0x01)];

        let display = SimulatorDisplay {
            size: Size::new(2, 1),
            pixels: expected.clone().into_boxed_slice(),
        };

        assert_eq!(
            &display.to_be_bytes(),
            &[0x80, 0x00, 0x00, 0x00, 0x00, 0x01]
        );
        assert_eq!(
            &display.to_le_bytes(),
            &[0x00, 0x00, 0x80, 0x01, 0x00, 0x00]
        );
    }

    #[test]
    fn diff_equal() {
        let display = SimulatorDisplay::<BinaryColor>::new(Size::new(4, 6));
        let expected = display.clone();

        assert_eq!(display.diff(&expected), None);
    }

    #[test]
    fn diff_not_equal() {
        let circle = Circle::new(Point::zero(), 3);

        let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(4, 6));
        let expected = display.clone();

        circle
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display)
            .unwrap();

        assert_eq!(display.diff(&expected), Some(display));
    }

    #[test]
    #[should_panic(expected = "both displays must have the same size (self: 4x6, other: 4x5)")]
    fn diff_wrong_size() {
        let display = SimulatorDisplay::<BinaryColor>::new(Size::new(4, 6));
        let expected = SimulatorDisplay::<BinaryColor>::new(Size::new(4, 5));

        assert_eq!(display.diff(&expected), None);
    }
}
