use std::io::{Error, ErrorKind, Read, Seek};
use std::io::BufReader;
use std::io::Result;

use crate::image_builder::{ImageBuilder, ImageBuilderFactory, ImageBuilderHelper};
use crate::ReadHelper;

const ISOMETRIC_TILE_WIDTH: u16 = 58;
const ISOMETRIC_TILE_HEIGHT: u16 = 30;
const ISOMETRIC_TILE_BYTES: u16 = 1800;
const ISOMETRIC_LARGE_TILE_WIDTH: u16 = 78;
const ISOMETRIC_LARGE_TILE_HEIGHT: u16 = 40;
const ISOMETRIC_LARGE_TILE_BYTES: u16 = 3200;

/// Metadata of an image.
///
/// Contains data about the type and dimensions of the image along with offsets of the pixel data.
///
/// Some bytes from the metadata are of unknown meaning.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SgImageMetadata {
    pub id: u32,
    pub offset: u32,
    pub length: u32,
    pub uncompressed_length: u32,
    pub zeroes: [u8; 4],
    pub invert_offset: i32,
    pub width: u16,
    pub height: u16,
    pub unknown_a: [u16; 3],
    pub anim_sprites: u16,
    pub unknown_b: u16,
    pub x_offset: u16,
    pub y_offset: u16,
    pub unknown_c: [u8; 10],
    pub is_reversible: u8,
    pub unknown_d: u8,
    pub image_type: u16,
    pub flags: [u8; 4],
    pub bitmap_id: u8,
    pub unknown_e: u8,
    pub anim_speed_id: u8,
    pub unknown_f: [u8; 5],
    pub alpha_offset: u32,
    pub alpha_length: u32,
}

impl SgImageMetadata {
    pub(crate) fn load<R: Read + Seek>(reader: &mut BufReader<R>, id: u32, include_alpha: bool) -> Result<SgImageMetadata> {
        let offset = reader.read_u32_le()?;
        let length = reader.read_u32_le()?;
        let uncompressed_length = reader.read_u32_le()?;
        let zeroes = reader.read_bytes()?;
        let invert_offset = reader.read_i32_le()?;
        let width = reader.read_u16_le()?;
        let height = reader.read_u16_le()?;
        let unknown_a = [reader.read_u16_le()?, reader.read_u16_le()?, reader.read_u16_le()?];
        let anim_sprites = reader.read_u16_le()?;
        let unknown_b = reader.read_u16_le()?;
        let x_offset = reader.read_u16_le()?;
        let y_offset = reader.read_u16_le()?;
        let unknown_c = reader.read_bytes()?;
        let is_reversible = reader.read_u8()?;
        let unknown_d = reader.read_u8()?;
        let image_type = reader.read_u16_le()?;
        let flags = reader.read_bytes()?;
        let bitmap_id = reader.read_u8()?;
        let unknown_e = reader.read_u8()?;
        let anim_speed_id = reader.read_u8()?;
        let unknown_f = reader.read_bytes()?;
        let alpha_offset = if include_alpha { reader.read_u32_le()? } else { 0 };
        let alpha_length = if include_alpha { reader.read_u32_le()? } else { 0 };

        let sg_image = SgImageMetadata {
            id,
            offset,
            length,
            uncompressed_length,
            zeroes,
            invert_offset,
            width,
            height,
            unknown_a,
            anim_sprites,
            unknown_b,
            x_offset,
            y_offset,
            unknown_c,
            is_reversible,
            unknown_d,
            image_type,
            flags,
            bitmap_id,
            unknown_e,
            anim_speed_id,
            unknown_f,
            alpha_offset,
            alpha_length,
        };

        return Ok(sg_image);
    }

    /// Checks if the image is flagged as having its data in an external file.
    pub fn is_external(&self) -> bool {
        return self.flags[0] > 0;
    }

    /// Load pixel data for this image from the provided reader.
    pub fn load_image<T, F: ImageBuilderFactory<T>, R: Read + Seek>(&self, reader: &mut BufReader<R>, image_builder_factory: &F) -> Result<T> {
        let current_position = reader.stream_position()?;

        let relative_position = self.offset as i64 - self.flags[0] as i64 - current_position as i64;

        if relative_position != 0 {
            reader.seek_relative(relative_position)?;
        }

        let mut image_builder = image_builder_factory.new_builder(self.width, self.height);

        if self.width <= 0 || self.height <= 0 || self.length <= 0 {
            return Ok(image_builder.build());
        }

        match self.image_type {
            0 | 1 | 10 | 12 | 13 => self.load_plain_image(&mut image_builder, reader)?,
            30 => self.load_isometric_image(&mut image_builder, reader)?,
            256 | 257 | 276 => self.load_sprite_image(&mut image_builder, reader)?,
            _ => return Err(Error::new(ErrorKind::Other, format!("Unrecognised image type: {}", self.image_type)))
        }

        if self.alpha_length > 0 {
            self.load_alpha_mask(&mut image_builder, reader)?;
        }

        if self.invert_offset != 0 {
            image_builder.flip_horizontal();
        }

        return Ok(image_builder.build());
    }

    fn load_plain_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut R) -> Result<()> {
        // Check image data
        if self.height as u32 * self.width as u32 * 2 != self.length {
            return Err(Error::new(ErrorKind::Other, "Image data length doesn't match image size"));
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let colour = reader.read_u16_le()?;
                image_builder.set_555_pixel(x, y, colour);
            }
        }

        return Ok(());
    }

    fn load_isometric_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut BufReader<R>) -> Result<()> {
        let current_position = reader.stream_position()?;

        let relative_position = self.offset as i64 - current_position as i64;

        if relative_position != 0 {
            reader.seek_relative(relative_position)?;
        }

        self.load_isometric_base(image_builder, reader)?;
        self.load_transparent_image(image_builder, reader, self.length - self.uncompressed_length, self.uncompressed_length + self.offset)?;
        Ok(())
    }

    fn load_isometric_base<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut R) -> Result<()> {
        let width = self.width;
        let height = (width + 2) / 2; // 58 -> 39, 118 -> 60 etc
        let size = self.calculate_isometric_size(height);
        let (_tile_bytes, tile_height, tile_width) = Self::calculate_tile_size(&size, &height);
        let height_offset = self.height - height;

        let mut y_offset = height_offset;

        if ((width + 2) * height) as u32 != self.uncompressed_length {
            return Err(Error::new(ErrorKind::Other, "Data length doesn't match footprint size"));
        }

        for y in 0..(size + size - 1) {
            let (x_lim, mut x_offset) = if y < size {
                (y + 1, (size - y - 1) * tile_height)
            } else {
                (2 * size - y - 1, (y - size + 1) * tile_height)
            };

            for _x in 0..x_lim {
                Self::write_isometric_tile(image_builder, reader, x_offset, y_offset, tile_width, tile_height)?;
                x_offset += tile_width + 2;
            }

            y_offset += tile_height / 2;
        }

        Ok(())
    }

    fn load_sprite_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut BufReader<R>) -> Result<()> {
        let current_position = reader.stream_position()?;

        let relative_position = self.offset as i64 - current_position as i64;

        if relative_position != 0 {
            reader.seek_relative(relative_position)?;
        }

        self.load_transparent_image(image_builder, reader, self.length, self.offset)?;
        Ok(())
    }

    fn calculate_isometric_size(&self, height: u16) -> u16 {
        if self.flags[3] == 0 {
            if (height % ISOMETRIC_TILE_HEIGHT) == 0 {
                return height / ISOMETRIC_TILE_HEIGHT;
            } else if (height % ISOMETRIC_LARGE_TILE_HEIGHT) == 0 {
                return height / ISOMETRIC_LARGE_TILE_HEIGHT;
            }
        }
        return self.flags[3] as u16;
    }

    fn write_isometric_tile<T, B: ImageBuilder<T>, R: Read + Seek>(image_builder: &mut B, reader: &mut R, offset_x: u16, offset_y: u16, tile_width: u16, tile_height: u16) -> Result<()> {
        let half_height = tile_height / 2;

        for y in 0..half_height {
            let start = tile_height - 2 * (y + 1);
            let end = tile_width - start;
            for x in start..end {
                let r = reader.read_u8()? as u16;
                let l = reader.read_u8()? as u16;
                image_builder.set_555_pixel(offset_x + x, offset_y + y, l << 8 | r);
            }
        }

        for y in half_height..tile_height {
            let start = 2 * y - tile_height;
            let end = tile_width - start;
            for x in start..end {
                let r = reader.read_u8()? as u16;
                let l = reader.read_u8()? as u16;
                image_builder.set_555_pixel(offset_x + x, offset_y + y, l << 8 | r);
            }
        }

        Ok(())
    }

    fn calculate_tile_size(size: &u16, height: &u16) -> (u16, u16, u16) {
        return if ISOMETRIC_TILE_HEIGHT * size == *height {
            (ISOMETRIC_TILE_BYTES, ISOMETRIC_TILE_HEIGHT, ISOMETRIC_TILE_WIDTH)
        } else {
            (ISOMETRIC_LARGE_TILE_BYTES, ISOMETRIC_LARGE_TILE_HEIGHT, ISOMETRIC_LARGE_TILE_WIDTH)
        }
    }

    fn load_transparent_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut BufReader<R>, length: u32, offset: u32) -> Result<()> {
        let mut x = 0;
        let mut y = 0;
        let width = self.width;

        while (reader.stream_position()? as u32) < length + offset {
            let c = reader.read_u8()?;

            if c == 255 {
                // The next number is pixels to skip
                x += reader.read_u8()? as u16;

                // TODO Change the while to mods and divides?
                while x >= width {
                    y += 1;
                    x -= width;
                }
            } else {
                // Pixels to fill in
                for _j in 0..c {
                    let r = reader.read_u8()? as u16;
                    let l = reader.read_u8()? as u16;
                    image_builder.set_555_pixel(x, y, l << 8 | r);
                    x += 1;
                    if x >= width {
                        y += 1;
                        x -= width;
                    }
                }
            }
        }

        Ok(())
    }

    fn load_alpha_mask<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut R) -> Result<()> {
        let mut x = 0;
        let mut y = 0;
        let width = self.width;

        while (reader.stream_position()? as u32) < (self.offset + self.length + self.alpha_length) {
            let c = reader.read_u8()?;

            if c == 255 {
                // The next number is pixels to skip
                x += reader.read_u8()? as u16;

                // Change the while to mods and divides
                while x >= width {
                    y += 1;
                    x -= width;
                }
            } else {
                // Pixels to fill in
                for _j in 0..c {
                    let alpha = reader.read_u8()?;
                    image_builder.set_alpha(x, y, alpha << 3);
                    x += 1;
                    if x >= width {
                        y += 1;
                        x = 0;
                    }
                }
            }
        }

        Ok(())
    }
}