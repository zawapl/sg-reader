use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, stdin};
use std::path::Path;

use sg_reader::{SgFileMetadata, VecImageBuilderFactory};

fn main() {
    let path = read_path().expect("Could not read user input");

    let sg_file = SgFileMetadata::load_metadata(Path::new(path.as_str())).expect("Failed to read metadata");

    println!("{:#?}", sg_file);

    let mut readers = HashMap::new();
    let mut image_by_type = HashMap::new();

    for image in &sg_file.images {
        let bitmap_id = image.bitmap_id as usize;

        if !readers.contains_key(&bitmap_id) {
            let path_buf = &sg_file.get_555_file_path(bitmap_id);
            let msg = format!("Could not open file {}", path_buf.display());
            let reader = BufReader::new(File::open(path_buf).expect(msg.as_str()));
            readers.insert(bitmap_id, reader);
        }

        let reader = readers.get_mut(&bitmap_id).unwrap();
        image.load_image(reader, &VecImageBuilderFactory).expect("Could not read image data");

        if !image_by_type.contains_key(&image.image_type) {
            image_by_type.insert(image.image_type, 1);
        } else {
            *image_by_type.get_mut(&image.image_type).unwrap() += 1;
        }
    }

    println!("Image count by type: {:?}", image_by_type);
}

fn read_path() -> Result<String> {
    let mut s= String::new();

    println!("Please, enter path to a sg3 file:");

    stdin().read_line(&mut s)?;

    return Ok(String::from(s.trim()));
}