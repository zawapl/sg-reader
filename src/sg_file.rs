use crate::ReadHelper;
use crate::Result;
use crate::*;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Metadata of a sg file.
///
/// Contains metadata of the images retrieved from the sg file.
/// Can be used to get information about the bitmaps and images this file describes.
///
/// Some bytes from the metadata are of unknown meaning.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SgFileMetadata {
    pub folder: String,
    pub filename: String,
    pub file_size: u32,
    pub version: u32,
    pub unknown: u32,
    pub max_image_count: u32,
    pub bitmap_records_without_system: u32,
    pub total_file_size: u32,
    pub file_size_555: u32,
    pub file_size_external: u32,
    pub bitmaps: Vec<SgBitmapMetadata>,
    pub images: Vec<SgImageMetadata>,
}

impl SgFileMetadata {
    /// Load metadata from provided reader
    pub fn load_metadata_from_reader<R: Read + Seek>(reader: &mut BufReader<R>, folder: String, filename: String) -> Result<Self> {
        let file_size = reader.read_u32_le()?;
        let version = reader.read_u32_le()?;

        let unknown = reader.read_u32_le()?;
        let max_image_count = reader.read_u32_le()?;
        let image_count = reader.read_u32_le()?;
        let bitmap_count = reader.read_u32_le()?;
        let bitmap_records_without_system = reader.read_u32_le()?;
        let total_file_size = reader.read_u32_le()?;
        let file_size_555 = reader.read_u32_le()?;
        let file_size_external = reader.read_u32_le()?;

        let max_bitmaps_records: u32 = if version == 0xd3 { 100 } else { 200 };

        reader.seek_relative(640)?;

        let bitmaps = Self::load_bitmaps_metadata(reader, bitmap_count)?;

        reader.seek_relative(200 * (max_bitmaps_records - bitmap_count) as i64)?;

        let images = Self::load_images_metadata(reader, image_count, version >= 0xd6)?;

        reader.seek(SeekFrom::End(0))?;

        Self::validate_header(&version, &file_size, &reader.stream_position()?)?;

        let sg_file = SgFileMetadata {
            folder,
            filename,
            file_size,
            version,
            unknown,
            max_image_count,
            bitmap_records_without_system,
            total_file_size,
            file_size_555,
            file_size_external,
            bitmaps,
            images,
        };

        Ok(sg_file)
    }

    /// Load metadata from the file founds on the given path.
    pub fn load_metadata_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);

        let folder = String::from(path.as_ref().parent().unwrap().to_str().unwrap());
        let filename = String::from(path.as_ref().file_name().unwrap().to_str().unwrap());

        Self::load_metadata_from_reader(&mut reader, folder, filename)
    }

    /// Load metadata and pixel data.
    pub fn load_fully<P: AsRef<Path>, T, F: ImageBuilderFactory<T>>(path: P, image_builder_factory: &F) -> Result<(Self, Vec<T>)> {
        let sg_file = Self::load_metadata_from_path(path)?;

        let images = sg_file.load_image_data(image_builder_factory)?;

        Ok((sg_file, images))
    }

    fn validate_header(version: &u32, file_size: &u32, actual_file_size: &u64) -> Result<()> {
        // SG2 file: FILE_SIZE = 74480 or 522680 (depending on whether it's a "normal" sg2 or an enemy sg2
        if version == &0xd3 && !(file_size == &74480 || file_size == &522680) {
            return Err(SgImageError::InvalidHeader);
        }

        // SG3 file: FILE_SIZE = the actual size of the sg3 file
        if (version == &0xd5 || version == &0xd6) && !(file_size == &74480 || actual_file_size == &(*file_size as u64)) {
            return Err(SgImageError::InvalidHeader);
        }

        Ok(())
    }

    fn load_bitmaps_metadata<R: Read + Seek>(reader: &mut BufReader<R>, bitmap_records: u32) -> Result<Vec<SgBitmapMetadata>> {
        let mut bitmaps = Vec::with_capacity(bitmap_records as usize);
        for i in 0..bitmap_records {
            bitmaps.push(SgBitmapMetadata::load(reader, i)?);
        }
        Ok(bitmaps)
    }

    fn load_images_metadata<R: Read + Seek>(file: &mut BufReader<R>, image_records: u32, alpha: bool) -> Result<Vec<SgImageMetadata>> {
        let mut images: Vec<SgImageMetadata> = Vec::with_capacity(image_records as usize);

        for i in 0..(image_records + 1) {
            let mut image = SgImageMetadata::load(file, i, alpha)?;

            let invert_offset = image.invert_offset;
            if invert_offset != 0 {
                image = images[(i as i32 + invert_offset) as usize].clone();
                image.id = i;
                image.invert_offset = invert_offset;
            }

            images.push(image);
        }

        Ok(images)
    }

    fn load_image_data<T, F: ImageBuilderFactory<T>>(&self, image_factory_builder: &F) -> Result<Vec<T>> {
        if self.images.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let mut last_file_params = (0, false);
        let path = self.get_555_file_path(0, false);
        let mut reader = BufReader::new(File::open(path)?);

        for i in 0..self.images.len() {
            let image = &self.images[i];
            let file_params = (image.bitmap_id as usize, image.is_external());

            if last_file_params != file_params {
                let path = self.get_555_file_path(file_params.0, file_params.1);
                reader = BufReader::new(File::open(path.clone())?);
                last_file_params = file_params;
            }

            result.push(image.load_image(&mut reader, image_factory_builder)?);
        }

        Ok(result)
    }

    /// Get path to the file containing pixel data for the given bitmap.
    pub fn get_555_file_path(&self, bitmap_id: usize, is_external: bool) -> PathBuf {
        let basename = if is_external {
            &self.bitmaps[bitmap_id].external_filename
        } else {
            &self.filename
        };

        let filename = format!("{}.555", &basename[..basename.len() - 4]);

        let path_buf: PathBuf = [&self.folder, &filename].iter().collect();

        path_buf
    }
}
