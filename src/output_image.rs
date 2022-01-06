use std::{convert::TryFrom, marker::PhantomData, path::Path};

use embedded_graphics::{
    pixelcolor::{raw::ToBytes, Gray8, Rgb888, RgbColor},
    prelude::*,
    primitives::Rectangle,
};
use image::{
    png::{CompressionType, FilterType, PngEncoder},
    ImageBuffer, Luma, Pixel as _, Rgb,
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
    pub(crate) output_settings: OutputSettings,
    color_type: PhantomData<C>,
}

impl<C> OutputImage<C>
where
    C: PixelColor + From<Rgb888> + ToBytes,
    <C as ToBytes>::Bytes: AsRef<[u8]>,
{
    /// Creates a new output image.
    pub(crate) fn new<DisplayC>(
        display: &SimulatorDisplay<DisplayC>,
        output_settings: &OutputSettings,
    ) -> Self
    where
        DisplayC: PixelColor + Into<Rgb888>,
    {
        let size = output_settings.framebuffer_size(display);

        // Create an empty pixel buffer, filled with the background color.
        let background_color = C::from(output_settings.theme.convert(Rgb888::BLACK)).to_be_bytes();
        let data = background_color
            .as_ref()
            .iter()
            .copied()
            .cycle()
            .take(size.width as usize * size.height as usize * background_color.as_ref().len())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            size,
            data,
            output_settings: output_settings.clone(),
            color_type: PhantomData,
        }
    }

    /// Updates the image from a [`SimulatorDisplay`].
    pub fn update<DisplayC>(&mut self, display: &SimulatorDisplay<DisplayC>)
    where
        DisplayC: PixelColor + Into<Rgb888>,
    {
        let pixel_pitch = (self.output_settings.scale + self.output_settings.pixel_spacing) as i32;
        let pixel_size = Size::new(self.output_settings.scale, self.output_settings.scale);

        for p in display.bounding_box().points() {
            let raw_color = display.get_pixel(p).into();
            let themed_color = self.output_settings.theme.convert(raw_color);
            let output_color = C::from(themed_color).to_be_bytes();
            let output_color = output_color.as_ref();

            for p in Rectangle::new(p * pixel_pitch, pixel_size).points() {
                if let Ok((x, y)) = <(u32, u32)>::try_from(p) {
                    let start_index = (x + y * self.size.width) as usize * output_color.len();

                    self.data[start_index..start_index + output_color.len()]
                        .copy_from_slice(output_color)
                }
            }
        }
    }
}

impl<C: OutputImageColor> OutputImage<C> {
    /// Saves the image content to a PNG file.
    pub fn save_png<PATH: AsRef<Path>>(&self, path: PATH) -> image::ImageResult<()> {
        let png = self.encode_png()?;

        std::fs::write(path, &png)?;

        Ok(())
    }

    /// Returns the image as a base64 encoded PNG.
    pub fn to_base64_png(&self) -> image::ImageResult<String> {
        let png = self.encode_png()?;

        Ok(base64::encode(&png))
    }

    fn encode_png(&self) -> image::ImageResult<Vec<u8>> {
        let mut png = Vec::new();

        PngEncoder::new_with_quality(&mut png, CompressionType::Best, FilterType::default())
            .encode(
                self.data.as_ref(),
                self.size.width,
                self.size.height,
                C::ImageColor::COLOR_TYPE,
            )?;

        Ok(png)
    }

    /// Returns the output image as an [`image`] crate [`ImageBuffer`].
    pub fn as_image_buffer(&self) -> ImageBuffer<C::ImageColor, &[u8]> {
        ImageBuffer::from_raw(self.size.width, self.size.height, self.data.as_ref()).unwrap()
    }
}

impl<C> OriginDimensions for OutputImage<C> {
    fn size(&self) -> Size {
        self.size
    }
}

pub trait OutputImageColor {
    type ImageColor: image::Pixel<Subpixel = u8> + 'static;
}

impl OutputImageColor for Gray8 {
    type ImageColor = Luma<u8>;
}

impl OutputImageColor for Rgb888 {
    type ImageColor = Rgb<u8>;
}
