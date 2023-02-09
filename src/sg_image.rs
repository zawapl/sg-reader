use crate::image_builder::{ImageBuilder, ImageBuilderFactory, ImageBuilderHelper};
use crate::Result;
use crate::{ReadHelper, SgImageError};
use std::io::BufReader;
use std::io::{Read, Seek};

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
        let alpha_offset = if include_alpha {
            reader.read_u32_le()?
        } else {
            0
        };
        let alpha_length = if include_alpha {
            reader.read_u32_le()?
        } else {
            0
        };

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
        let mut image_builder = image_builder_factory.new_builder(self.width, self.height);

        if self.width <= 0 || self.height <= 0 || self.length <= 0 {
            return Ok(image_builder.build());
        }

        match self.image_type {
            0 | 1 | 10 | 12 | 13 => self.load_plain_image(&mut image_builder, reader)?,
            30 => self.load_isometric_image(&mut image_builder, reader)?,
            256 | 257 | 276 => self.load_sprite_image(&mut image_builder, reader)?,
            _ => return Err(SgImageError::UnknownImageType(self.image_type)),
        }

        if self.alpha_length > 0 {
            self.load_alpha_mask(&mut image_builder, reader)?;
        }

        if self.invert_offset != 0 {
            image_builder.flip_horizontal();
        }

        return Ok(image_builder.build());
    }

    fn load_plain_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut BufReader<R>) -> Result<()> {
        let current_position = reader.stream_position()?;

        let relative_position = self.offset as i64 - self.flags[0] as i64 - current_position as i64;

        if relative_position != 0 {
            reader.seek_relative(relative_position)?;
        }

        // Check image data
        if self.height as u32 * self.width as u32 * 2 != self.length {
            return Err(SgImageError::ImageDataLengthMismatch);
        }

        for position in 0..(self.length as usize) / 2 {
            let colour = reader.read_u16_le()?;
            image_builder.set_555_pixel_by_pos(position, colour);
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
        self.load_transparent_image(image_builder, reader, &(self.length - self.uncompressed_length))?;

        Ok(())
    }

    fn load_isometric_base<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut R) -> Result<()> {
        let width = self.width;
        let height = (width + 2) / 2; // 58 -> 39, 118 -> 60 etc
        let size = self.calculate_isometric_size(height);
        let (_tile_bytes, tile_height, tile_width) = Self::calculate_tile_size(&size, &height);
        let height_offset = self.height - height;

        let mut y_offset = height_offset;

        if ((width as u32 + 2) * height as u32) != self.uncompressed_length {
            return Err(SgImageError::ImageDataLengthMismatch);
        }

        for y in 0..(size + size - 1) {
            let (x_lim, mut x_offset) = if y < size {
                (y + 1, (size - y - 1) * tile_height)
            } else {
                (2 * size - y - 1, (y - size + 1) * tile_height)
            };

            for _x in 0..x_lim {
                self.write_isometric_tile(image_builder, reader, x_offset as usize, y_offset as usize, tile_width as usize, tile_height as usize)?;
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

        self.load_transparent_image(image_builder, reader, &self.length)?;
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

    fn write_isometric_tile<T, B: ImageBuilder<T>, R: Read + Seek>(
        &self,
        image_builder: &mut B,
        reader: &mut R,
        offset_x: usize,
        offset_y: usize,
        tile_width: usize,
        tile_height: usize,
    ) -> Result<()> {
        let half_height = (tile_height / 2) as usize;

        let mut x_start = tile_height;
        let mut x_end = tile_width - x_start;
        let mut position = offset_x + (offset_y * self.width as usize);
        let skip = (self.width as usize) - tile_width;

        for _y in 0..half_height {
            x_start -= 2;
            x_end += 2;
            position += x_start;
            for _x in x_start..x_end {
                let c = reader.read_u16_le()?;
                image_builder.set_555_pixel_by_pos(position, c);
                position += 1;
            }
            position += x_start + skip;
        }

        for _y in half_height..tile_height {
            position += x_start;
            for _x in x_start..x_end {
                let c = reader.read_u16_le()?;
                image_builder.set_555_pixel_by_pos(position, c);
                position += 1;
            }
            position += x_start + skip;
            x_start += 2;
            x_end -= 2;
        }

        Ok(())
    }

    fn calculate_tile_size(size: &u16, height: &u16) -> (u16, u16, u16) {
        return if ISOMETRIC_TILE_HEIGHT * size == *height {
            (ISOMETRIC_TILE_BYTES, ISOMETRIC_TILE_HEIGHT, ISOMETRIC_TILE_WIDTH)
        } else {
            (ISOMETRIC_LARGE_TILE_BYTES, ISOMETRIC_LARGE_TILE_HEIGHT, ISOMETRIC_LARGE_TILE_WIDTH)
        };
    }

    fn load_transparent_image<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut BufReader<R>, length: &u32) -> Result<()> {
        let mut pos = 0;
        let mut remaining_bytes = *length as usize;

        while remaining_bytes > 0 {
            let c = reader.read_u8()? as usize;

            if c == 255 {
                // The next number is pixels to skip
                pos += reader.read_u8()? as usize;
                remaining_bytes -= 2;
            } else {
                remaining_bytes -= 1 + (c * 2);
                // Pixels to fill in
                for _j in 0..c {
                    let pixel = reader.read_u16_le()?;
                    image_builder.set_555_pixel_by_pos(pos, pixel);
                    pos += 1;
                }
            }
        }

        Ok(())
    }

    fn load_alpha_mask<T, B: ImageBuilder<T>, R: Read + Seek>(&self, image_builder: &mut B, reader: &mut R) -> Result<()> {
        let mut pos = 0;
        let mut remaining_bytes = self.alpha_length as usize;

        while remaining_bytes > 0 {
            let c = reader.read_u8()? as usize;

            if c == 255 {
                // The next number is pixels to skip
                pos += reader.read_u8()? as usize;
                remaining_bytes -= 2;
            } else {
                // Pixels to fill in
                remaining_bytes -= 1 + c;
                for _j in 0..c {
                    let alpha = reader.read_u8()?;
                    image_builder.set_alpha(pos, alpha << 3);
                    pos += 1;
                }
            }
        }

        Ok(())
    }
}
