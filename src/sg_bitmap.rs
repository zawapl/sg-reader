use crate::utils::ReadHelper;
use crate::Result;
use std::io::{BufReader, Read, Seek};
use std::string::String;

/// Metadata of a bitmap.
///
/// In this context a bitmap is a group of images sharing the same file containing their pixel data.
///
/// Some bytes from the metadata are of unknown meaning.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SgBitmapMetadata {
    pub id: u32,
    pub external_filename: String,
    pub comment: String,
    pub width: u32,
    pub height: u32,
    pub num_images: u32,
    pub start_index: u32,
    pub end_index: u32,
    pub image_id: u32,           // u32 between start & end - id of an image?
    pub unknown_a: u32,          // unknown purpose
    pub unknown_b: u32,          // unknown purpose
    pub unknown_c: u32,          // unknown purpose
    pub unknown_d: u32,          // unknown purpose
    pub image_width: u32,        // real width? - corresponding to image width
    pub image_height: u32,       // real height? - corresponding to image height
    pub file_size_555: u32,      // if non-zero -> internal image
    pub total_file_size: u32,    // if non-zero -> internal image
    pub file_size_external: u32, // if non-zero -> internal image
    pub unknown_e: [u8; 24],     // 24 unknown bytes
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
        let unknown_a = reader.read_u32_le()?;
        let unknown_b = reader.read_u32_le()?;
        let unknown_c = reader.read_u32_le()?;
        let unknown_d = reader.read_u32_le()?;
        let image_width = reader.read_u32_le()?;
        let image_height = reader.read_u32_le()?;
        let file_size_555 = reader.read_u32_le()?;
        let total_file_size = reader.read_u32_le()?;
        let file_size_external = reader.read_u32_le()?;
        let unknown_e = reader.read_bytes()?;

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
            unknown_a,
            unknown_b,
            unknown_c,
            unknown_d,
            image_width,
            image_height,
            file_size_555,
            total_file_size,
            file_size_external,
            unknown_e,
        };

        return Ok(sg_bitmap_metadata);
    }
}
