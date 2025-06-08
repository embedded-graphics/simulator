use std::{convert::TryFrom, marker::PhantomData, path::Path};

use base64::Engine;
use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Gray8, Rgb888},
    prelude::*,
    primitives::Rectangle,
};
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ImageBuffer, ImageEncoder, Luma, Rgb,
};

use crate::{display::SimulatorDisplay, output_settings::OutputSettings};

/// Output image.
///
/// An output image is the result of applying [`OutputSettings`] to a [`SimulatorDisplay`]. It can
/// be used to save a simulator display to a PNG file.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutputImage<C> {
    size: Size,
    pub(crate) data: Box<[u8]>,
    row_buffer: Vec<u8>,

    color_type: PhantomData<C>,
}

impl<C> OutputImage<C>
where
    C: PixelColor + OutputImageColor + From<Rgb888>,
    Self: DrawTarget<Color = C, Error = ()>,
{
    /// Creates a new output image.
    pub(crate) fn new(size: Size) -> Self {
        let bytes_per_row = usize::try_from(size.width).unwrap() * C::BYTES_PER_PIXEL;
        let bytes_total = usize::try_from(size.height).unwrap() * bytes_per_row;

        let data = vec![0; bytes_total].into_boxed_slice();
        let row_buffer = Vec::with_capacity(bytes_per_row);

        Self {
            size,
            data,
            row_buffer,
            color_type: PhantomData,
        }
    }

    /// Draws a display using the given position and output setting.
    pub fn draw_display<DisplayC>(
        &mut self,
        display: &SimulatorDisplay<DisplayC>,
        position: Point,
        output_settings: &OutputSettings,
    ) where
        DisplayC: PixelColor + Into<Rgb888>,
    {
        let display_area = Rectangle::new(position, display.output_size(output_settings));
        self.fill_solid(
            &display_area,
            output_settings.theme.convert(Rgb888::BLACK).into(),
        )
        .unwrap();

        if output_settings.scale == 1 {
            display
                .bounding_box()
                .points()
                .map(|p| {
                    let raw_color = display.get_pixel(p).into();
                    let themed_color = output_settings.theme.convert(raw_color);
                    let output_color = C::from(themed_color);

                    Pixel(p + position, output_color)
                })
                .draw(self)
                .unwrap();
        } else {
            let pixel_pitch = (output_settings.scale + output_settings.pixel_spacing) as i32;
            let pixel_size = Size::new(output_settings.scale, output_settings.scale);

            for p in display.bounding_box().points() {
                let raw_color = display.get_pixel(p).into();
                let themed_color = output_settings.theme.convert(raw_color);
                let output_color = C::from(themed_color);

                self.fill_solid(
                    &Rectangle::new(p * pixel_pitch + position, pixel_size),
                    output_color,
                )
                .unwrap();
            }
        }
    }
}

impl<C: OutputImageColor> OutputImage<C> {
    /// Saves the image content to a PNG file.
    pub fn save_png<PATH: AsRef<Path>>(&self, path: PATH) -> image::ImageResult<()> {
        let png = self.encode_png()?;

        std::fs::write(path, png)?;

        Ok(())
    }

    /// Returns the image as a base64 encoded PNG.
    pub fn to_base64_png(&self) -> image::ImageResult<String> {
        let png = self.encode_png()?;

        Ok(base64::engine::general_purpose::STANDARD.encode(png))
    }

    fn encode_png(&self) -> image::ImageResult<Vec<u8>> {
        let mut png = Vec::new();

        PngEncoder::new_with_quality(&mut png, CompressionType::Best, FilterType::default())
            .write_image(
                self.data.as_ref(),
                self.size.width,
                self.size.height,
                C::IMAGE_COLOR_TYPE.into(),
            )?;

        Ok(png)
    }

    /// Returns the output image as an [`image`] crate [`ImageBuffer`].
    pub fn as_image_buffer(&self) -> ImageBuffer<C::ImageColor, &[u8]> {
        ImageBuffer::from_raw(self.size.width, self.size.height, self.data.as_ref()).unwrap()
    }
}

impl DrawTarget for OutputImage<Rgb888> {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, color) in pixels {
            if p.x >= 0
                && p.y >= 0
                && (p.x as u32) < self.size.width
                && (p.y as u32) < self.size.height
            {
                let bytes = color.to_be_bytes();
                let (x, y) = (p.x as u32, p.y as u32);

                let start_index = (x + y * self.size.width) as usize * 3;
                self.data[start_index..start_index + 3].copy_from_slice(bytes.as_ref())
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());

        let bytes = color.to_be_bytes();
        let bytes = bytes.as_ref();

        // For large areas it's more efficient to prepare a row buffer and copy
        // the entire row at one.
        // TODO: the bounds were chosen arbitrarily and might not be optimal
        let large = area.size.width >= 16 && area.size.height >= 16;

        if large {
            self.row_buffer.clear();
            for _ in 0..area.size.width {
                self.row_buffer.extend_from_slice(bytes);
            }
        }

        let bytes_per_row = self.size.width as usize * bytes.len();
        let x_start = area.top_left.x as usize * bytes.len();
        let x_end = x_start + area.size.width as usize * bytes.len();

        if large {
            for y in area.rows() {
                let start = bytes_per_row * y as usize + x_start;
                self.data[start..start + self.row_buffer.len()].copy_from_slice(&self.row_buffer);
            }
        } else {
            for y in area.rows() {
                let row_start = bytes_per_row * y as usize;
                for chunk in
                    self.data[row_start + x_start..row_start + x_end].chunks_exact_mut(bytes.len())
                {
                    chunk.copy_from_slice(bytes);
                }
            }
        }

        Ok(())
    }
}

impl DrawTarget for OutputImage<Gray8> {
    type Color = Gray8;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, color) in pixels {
            if p.x >= 0
                && p.y >= 0
                && (p.x as u32) < self.size.width
                && (p.y as u32) < self.size.height
            {
                let (x, y) = (p.x as u32, p.y as u32);
                let index = (x + y * self.size.width) as usize;
                self.data[index] = color.into_storage();
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());

        let bytes_per_row = self.size.width as usize;
        let x_start = area.top_left.x as usize;
        let x_end = x_start + area.size.width as usize;

        for y in area.rows() {
            let row_start = bytes_per_row * y as usize;
            self.data[row_start + x_start..row_start + x_end].fill(color.into_storage());
        }

        Ok(())
    }
}

impl<C> OriginDimensions for OutputImage<C> {
    fn size(&self) -> Size {
        self.size
    }
}

pub trait OutputImageColor {
    type ImageColor: image::Pixel<Subpixel = u8> + 'static;
    const IMAGE_COLOR_TYPE: image::ColorType;
    const BYTES_PER_PIXEL: usize;
}

impl OutputImageColor for Gray8 {
    type ImageColor = Luma<u8>;
    const IMAGE_COLOR_TYPE: image::ColorType = image::ColorType::L8;
    const BYTES_PER_PIXEL: usize = 1;
}

impl OutputImageColor for Rgb888 {
    type ImageColor = Rgb<u8>;
    const IMAGE_COLOR_TYPE: image::ColorType = image::ColorType::Rgb8;
    const BYTES_PER_PIXEL: usize = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb888_default_data() {
        let image = OutputImage::<Rgb888>::new(Size::new(6, 5));
        assert_eq!(image.data.as_ref(), &[0u8; 6 * 5 * 3]);
    }

    #[test]
    fn rgb888_draw_iter() {
        let mut image = OutputImage::<Rgb888>::new(Size::new(4, 6));

        [
            Pixel(Point::new(0, 0), Rgb888::new(0xFF, 0x00, 0x00)),
            Pixel(Point::new(3, 0), Rgb888::new(0x00, 0xFF, 0x00)),
            Pixel(Point::new(0, 5), Rgb888::new(0x00, 0x00, 0xFF)),
            Pixel(Point::new(3, 5), Rgb888::new(0x12, 0x34, 0x56)),
            // out of bounds pixels should be ignored
            Pixel(Point::new(-1, -1), Rgb888::new(0xFF, 0xFF, 0xFF)),
            Pixel(Point::new(0, 10), Rgb888::new(0xFF, 0xFF, 0xFF)),
            Pixel(Point::new(10, 0), Rgb888::new(0xFF, 0xFF, 0xFF)),
        ]
        .into_iter()
        .draw(&mut image)
        .unwrap();

        assert_eq!(
            image.data.as_ref(),
            &[
                0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, //
            ]
        );
    }

    #[test]
    fn rgb888_fill_solid() {
        let mut image = OutputImage::<Rgb888>::new(Size::new(4, 6));

        image
            .fill_solid(
                &Rectangle::new(Point::new(2, 3), Size::new(10, 20)),
                Rgb888::new(0x01, 0x02, 0x03),
            )
            .unwrap();

        assert_eq!(
            image.data.as_ref(),
            &[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x01, 0x02, 0x03, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x01, 0x02, 0x03, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x01, 0x02, 0x03, //
            ]
        );
    }

    #[test]
    fn gray8_default_data() {
        let image = OutputImage::<Gray8>::new(Size::new(6, 5));
        assert_eq!(image.data.as_ref(), &[0u8; 6 * 5]);
    }

    #[test]
    fn gray8_draw_iter() {
        let mut image = OutputImage::<Gray8>::new(Size::new(12, 6));

        [
            Pixel(Point::new(0, 0), Gray8::new(0x01)),
            Pixel(Point::new(11, 0), Gray8::new(0x02)),
            Pixel(Point::new(0, 5), Gray8::new(0x03)),
            Pixel(Point::new(11, 5), Gray8::new(0x04)),
            // out of bounds pixels should be ignored
            Pixel(Point::new(-1, -1), Gray8::new(0xFF)),
            Pixel(Point::new(0, 10), Gray8::new(0xFF)),
            Pixel(Point::new(12, 0), Gray8::new(0xFF)),
        ]
        .into_iter()
        .draw(&mut image)
        .unwrap();

        assert_eq!(
            image.data.as_ref(),
            &[
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, //
            ]
        );
    }

    #[test]
    fn gray8_fill_solid() {
        let mut image = OutputImage::<Gray8>::new(Size::new(4, 6));

        image
            .fill_solid(
                &Rectangle::new(Point::new(2, 3), Size::new(10, 20)),
                Gray8::WHITE,
            )
            .unwrap();

        assert_eq!(
            image.data.as_ref(),
            &[
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0xFF, 0xFF, //
                0x00, 0x00, 0xFF, 0xFF, //
                0x00, 0x00, 0xFF, 0xFF, //
            ]
        );
    }
}
