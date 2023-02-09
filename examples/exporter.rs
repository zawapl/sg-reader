use image::ColorType;
use sg_image_reader::{SgFileMetadata, VecImageBuilderFactory};
use std::fs;
use std::io::{stdin, Error, ErrorKind, Result};
use std::path::PathBuf;
use std::time::Instant;

fn run() -> Result<()> {
    let mut s = String::new();

    println!("Please, enter folder to a unpack:");

    stdin().read_line(&mut s)?;

    let paths = fs::read_dir(s.trim()).unwrap();

    if let Err(_) = fs::remove_dir_all("./unpacked") {
        println!("Failed to delete target folder");
    }

    for path in paths {
        if let Ok(dir) = path {
            if dir.path().as_path().extension().map_or_else(|| false, |ext| ext.eq("sg3")) {
                println!("Unpacking {:?}", dir.path());
                let start = Instant::now();
                let mut path_buf = PathBuf::new();
                path_buf.push("./unpacked");
                path_buf.push(dir.file_name());
                fs::create_dir_all(path_buf.clone())?;

                match SgFileMetadata::load_fully(dir.path(), &VecImageBuilderFactory) {
                    Err(err) => println!("Failed to load: {:?}", err),
                    Ok((sg_file, pixels)) => {
                        for i in 0..pixels.len() {
                            let image = &sg_file.images[i];
                            let width = image.width.clone() as u32;
                            let height = image.height.clone() as u32;

                            if width == 0 || height == 0 {
                                continue;
                            }

                            let mut file_path = path_buf.clone();
                            file_path.push(format!("{}", i));
                            file_path.set_extension("png");

                            let result = image::save_buffer(file_path, &pixels[i], width, height, ColorType::Rgba8);
                            result.map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;
                        }
                    }
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
