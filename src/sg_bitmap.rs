use std::io::{BufReader, Read, Seek};
use std::io::Result;
use std::string::String;

use crate::utils::ReadHelper;

/// Metadata of a bitmap.
///
/// In this context a bitmap is a collection of images (terminology copied from other implementations).
///
/// All images in a bitmap have their data stored in the same file.
///
/// Some bytes from the metadata are unknown and are omitted from the struct.
///
#[derive(Debug, Clone)]
pub struct SgBitmapMetadata {
    pub id: u32,
    pub external_filename: String,
    pub comment: String,
    pub width: u32,
    pub height: u32,
    pub num_images: u32,
    pub start_index: u32,
    pub end_index: u32,
    pub image_id: u32, // u32 between start & end - id of an image?
    // pub unknown1: u32, // unknown purpose
    // pub unknown2: u32, // unknown purpose
    // pub unknown3: u32, // unknown purpose
    // pub unknown4: u32, // unknown purpose
    pub image_width: u32, // real width? - correspnding to image width
    pub image_height: u32, // real height? - corresponding to image height
    pub file_size_555: u32, // if non-zero -> internal image
    pub total_file_size: u32, // if non-zero -> internal image
    pub file_size_external: u32,// if non-zero -> internal image
    // 24 unknown bytes
}

impl SgBitmapMetadata {
    pub(crate) fn load<R: Read + Seek>(reader: &mut BufReader<R>, id: u32) -> Result<SgBitmapMetadata> {
        let external_filename = reader.read_utf(65)?;
        let comment = reader.read_utf(51)?;
        let width = reader.read_u32_le()?;
        let height = reader.read_u32_le()?;
        let num_images = reader.read_u32_le()?;
        let start_index = reader.read_u32_le()?;
        let end_index = reader.read_u32_le()?;
        let image_id = reader.read_u32_le()?;
        let _unknown1 = reader.read_u32_le()?;
        let _unknown2 = reader.read_u32_le()?;
        let _unknown3 = reader.read_u32_le()?;
        let _unknown4 = reader.read_u32_le()?;
        let image_width = reader.read_u32_le()?;
        let image_height = reader.read_u32_le()?;
        let file_size_555 = reader.read_u32_le()?;
        let total_file_size = reader.read_u32_le()?;
        let file_size_external = reader.read_u32_le()?;

        reader.seek_relative(24)?;

        let sg_bitmap_metadata = SgBitmapMetadata {
            id,
            external_filename,
            comment,
            width,
            height,
            num_images,
            start_index,
            end_index,
            image_id,
            image_width,
            image_height,
            file_size_555,
            total_file_size,
            file_size_external,
        };

        return Ok(sg_bitmap_metadata);
    }
}