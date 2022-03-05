use std::io::{Result, stdin};

use sg_image_reader::{SgFileMetadata, VecImageBuilderFactory};

fn run_example() -> Result<()> {
    let mut s = String::new();

    println!("Please, enter path to a sg3 file:");

    stdin().read_line(&mut s)?;

    let path = String::from(s.trim()).replace("\"", "");

    println!("Reading {}", path);


    let (_sg_file, _pixel_data) = SgFileMetadata::load_fully(path, &VecImageBuilderFactory)?;

    // println!("{:#?}", sg_file);

    return Ok(());
}

fn main() {
    run_example().expect("Failed to run the example");
}