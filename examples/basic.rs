use sg_image_reader::Result;
use sg_image_reader::{SgFileMetadata, VecImageBuilderFactory};
use std::io::stdin;
use std::time::Instant;

fn run_example() -> Result<()> {
    let mut s = String::new();

    println!("Please, enter path to a sg3 file:");

    stdin().read_line(&mut s)?;

    let path = String::from(s.trim()).replace("\"", "");

    println!("Reading {}", path);

    let start = Instant::now();

    let (_sg_file, _pixel_data) = SgFileMetadata::load_fully(path, &VecImageBuilderFactory)?;

    let elapsed_time = start.elapsed();
    println!("Finished in {}ms", elapsed_time.as_millis());

    return Ok(());
}

fn main() {
    run_example().expect("Failed to run the example");
}
