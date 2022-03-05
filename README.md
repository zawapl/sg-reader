# sg_image_reader

[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/zawapl/sg-reader/blob/main/LICENSE)

A library for reading sg3 files used in some Impressions Games city building games (Cesar 3, Pharaoh, Zeus, Emperor etc.).

Documentation of the format can be found at https://github.com/bvschaik/citybuilding-tools/wiki/SG-file-format#image-data.

Basic usage:
```rust
let path = "path-to-file";
let (sg_file, pixel_data): (SgFileMetadata, Vec<Vec<u8>>) = SgFileMetadata::load_fully(path, &VecImageBuilderFactory)?;
```

The basic example provides a vector of raw bytes for all the images.
The raw bytes can be used to construct required image structs (with the image library of your choosing).
It is also possible to construct the required images directly by implementing the `ImageBuilderFactory` trait and passing it instead of the `VecImageBuilderFactory`.

Pixel data can also be loaded for one image at a time, see `viewer` example for an example of that
```rust
// Load just the metadata
let sg_file = SgFileMetadata::load_metadata(path)?;

// Image we want to load pixel data for
let image = &sg_file.images[11];

// Get the path of the file where that data is located
let path = sg_file.get_555_file_path(image.bitmap_id as usize, image.is_external());

// Create a new reader
let mut buf_reader = BufReader::new(File::open(path)?);

// Load pixel data for that specific image
let pixel_data = image.load_image(&mut buf_reader, &VecImageBuilderFactory);
```

## Viewer example

You can run `cargo run --example viewer` to open up an example application for viewing the files:

![iso](screenshots/viewer.png)

---

Tested with Zeus, Pharaoh and Emperor files.