/// A trait for providing [ImageBuilder] for a new image.
///
/// Gets called internally to create a new builder for each encountered image.
///
/// See [VecImageBuilderFactory] for a version that creates a vector of bytes representing raw pixel data.
pub trait ImageBuilderFactory<T> {
    /// The type of the corresponding builder
    type Builder: ImageBuilder<T>;

    /// Create a new builder for an image of the provided size
    fn new_builder(&self, width: u16, height: u16) -> Self::Builder;
}

/// A trait for building an image from provided pixels.
///
/// The builder is used internally to set the pixels to the right values.
///
/// See [VecImageBuilder] for basic implementation that creates a vector of bytes representing raw pixel data.
pub trait ImageBuilder<T> {
    /// Set the specified pixel to the given colour given as RGBA
    fn set_pixel(&mut self, x: u16, y: u16, pixel: [u8; 4]);

    /// Set alpha on the specified pixel
    fn set_alpha(&mut self, x: u16, y: u16, alpha: u8);

    /// Mirror each pixel horizontally
    fn flip_horizontal(&mut self);

    /// Consume the builder and return the resulting image
    fn build(self) -> T;
}

/// Default implementation of [ImageBuilderFactory] for creating images as vector of bytes.
pub struct VecImageBuilderFactory;

impl ImageBuilderFactory<Vec<u8>> for VecImageBuilderFactory {
    type Builder = VecImageBuilder;

    fn new_builder(&self, width: u16, height: u16) -> Self::Builder {
        let width = width as usize;
        let height = height as usize;
        let pixels = vec![0; width * height * 4];
        return VecImageBuilder {
            width,
            height,
            pixels,
        };
    }
}

/// Default implementation of [ImageBuilder] for creating images as a vector of bytes.
pub struct VecImageBuilder {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl ImageBuilder<Vec<u8>> for VecImageBuilder {
    fn set_pixel(&mut self, x: u16, y: u16, pixel: [u8; 4]) {
        let i = (x as usize + (self.width * y as usize)) * 4;
        self.pixels[i..(i + 4)].clone_from_slice(&pixel);
    }

    fn set_alpha(&mut self, x: u16, y: u16, alpha: u8) {
        let i = (x as usize + (self.width * y as usize)) * 4 + 3;
        self.pixels[i] = alpha;
    }

    fn flip_horizontal(&mut self) {
        let mut row_offset = 0;
        for _y in 0..self.height {
            for x in 0..(self.width / 2) {
                let pixel_a = (row_offset + x) * 4;
                let pixel_b = (row_offset + self.width - x - 1) * 4;
                self.pixels.swap(pixel_a, pixel_b);
                self.pixels.swap(pixel_a + 1, pixel_b + 1);
                self.pixels.swap(pixel_a + 2, pixel_b + 2);
                self.pixels.swap(pixel_a + 3, pixel_b + 3);
            }
            row_offset += self.width;
        }
    }

    fn build(self) -> Vec<u8> {
        return self.pixels;
    }
}

pub(crate) trait ImageBuilderHelper<T> {
    fn set_555_pixel(&mut self, x: u16, y: u16, colour: u16);
}

impl<T, B: ImageBuilder<T>> ImageBuilderHelper<T> for B {
    fn set_555_pixel(&mut self, x: u16, y: u16, colour: u16) {
        if colour == 0xf81f {
            return;
        }

        let ones = 0xf8 as u8;
        let r = (colour >> 7) as u8 & ones;
        let g = (colour >> 2) as u8 & ones;
        let b = (colour << 3) as u8 & ones;

        let data = [r, g, b, 0xff];

        self.set_pixel(x, y, data);
    }
}
