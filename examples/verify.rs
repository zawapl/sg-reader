use sg_image_reader::{SgFileMetadata, VecImageBuilderFactory};
use std::fs;
use std::io::{stdin, Result};
use std::time::Instant;

fn run() -> Result<()> {
    let mut s = String::new();

    println!("Please, enter folder to a scan:");

    stdin().read_line(&mut s)?;

    let paths = fs::read_dir(s.trim()).unwrap();

    for path in paths {
        if let Ok(dir) = path {
            if dir.path().as_path().extension().map_or_else(|| false, |ext| ext.eq("sg3")) {
                println!("Verifying {:?}", dir.path());
                let start = Instant::now();
                if let Err(err) = SgFileMetadata::load_fully(dir.path(), &VecImageBuilderFactory) {
                    println!("Failed to load: {:?}", err);
                };
                let elapsed_time = start.elapsed();
                println!("Finished in {}ms", elapsed_time.as_millis());
            }
        }
    }

    return Ok(());
}

fn main() {
    run().expect("Failed to run the example");
}
