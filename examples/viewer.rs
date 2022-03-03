extern crate druid;
extern crate piet_common;

use std::fs::File;
use std::io::BufReader;
use std::vec::Vec;

use druid::*;
use druid::im::Vector;
use druid::widget::{Button, Container, Flex, Image, Label, List, Scroll, Split, ViewSwitcher};

use sg_image_reader::{SgFileMetadata, VecImageBuilderFactory};

use crate::piet::ImageFormat;

#[derive(Clone)]
struct LoadedFile(SgFileMetadata);

#[derive(Clone, Data, Lens, Default)]
struct AppData {
    title: String,
    images: Vector<(u32, String)>,
    current_image: Option<usize>,
    loaded_file: Option<LoadedFile>,
    pixels: Vector<u8>,
}

struct Delegate;

pub const SELECT_IMAGE: Selector<u32> = Selector::new("select-sg-image");

fn build_app() -> impl Widget<AppData> {
    // Create the button to open a filk
    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("SG3", &["sg3"])]);

    let left = Flex::column()
        .with_child(Button::new("Open file").on_click(move |ctx, _, _| {
            ctx.submit_command(Command::new(
                druid::commands::SHOW_OPEN_PANEL,
                open_dialog_options.clone(),
                Target::Auto,
            ))
        }).expand_width())
        .with_default_spacer()
        .with_flex_child(Scroll::new(List::new(|| {
            Button::new(|item: &(u32, String), _env: &_| item.1.clone())
                .on_click(|ctx, data, _| {
                    ctx.submit_command(Command::new(
                        SELECT_IMAGE,
                        data.0,
                        Target::Auto,
                    ))
                })
                .expand_width()
        })).vertical().lens(AppData::images).expand(), 1.0, );

    // Table with image details
    let table = ViewSwitcher::new(|data: &AppData, _env| data.current_image, move |current_image, data: &AppData, _env| {
        return Box::new(current_image.map_or_else(|| {
            return Scroll::new(Split::columns(Label::new(""), Label::new("")).split_point(0.5).draggable(false)).vertical();
        }, |image_id| {
            if let Some(LoadedFile(file)) = &data.loaded_file {
                let image = &file.images[image_id];
                let bitmap = &file.bitmaps[image.bitmap_id as usize];

                let mut labels = Flex::column();
                let mut values = Flex::column();

                let mut add_row = |label: &str, value: String| {
                    labels.add_child(Label::new(label).expand_width());
                    values.add_child(Label::new(value).expand_width());
                };

                add_row("image id", format!("{:?}", image.id));
                add_row("offset", format!("{:?}", image.offset));
                add_row("length", format!("{:?}", image.length));
                add_row("uncompressed_length", format!("{:?}", image.uncompressed_length));
                add_row("zeroes", format!("{:?}", image.zeroes));
                add_row("invert_offset", format!("{:?}", image.invert_offset));
                add_row("width", format!("{:?}", image.width));
                add_row("height", format!("{:?}", image.height));
                add_row("anim_sprites", format!("{:?}", image.anim_sprites));
                add_row("x_offset", format!("{:?}", image.x_offset));
                add_row("y_offset", format!("{:?}", image.y_offset));
                add_row("is_reversible", format!("{:?}", image.is_reversible));
                add_row("image_type", format!("{:?}", image.image_type));
                add_row("flags", format!("{:?}", image.flags));
                add_row("anim_speed_id", format!("{:?}", image.anim_speed_id));
                add_row("alpha_offset", format!("{:?}", image.alpha_offset));
                add_row("alpha_length", format!("{:?}", image.alpha_length));
                add_row("unknown_a", format!("{:?}", image.unknown_a));
                add_row("unknown_b", format!("{:?}", image.unknown_b));
                add_row("unknown_c", format!("{:?}", image.unknown_c));
                add_row("unknown_d", format!("{:?}", image.unknown_d));
                add_row("unknown_e", format!("{:?}", image.unknown_e));
                add_row("unknown_f", format!("{:?}", image.unknown_f));
                add_row("", format!(" "));
                add_row("bitmap_id", format!("{:?}", bitmap.id));
                add_row("external_filename", format!("{:?}", bitmap.external_filename));
                add_row("comment", format!("{:?}", bitmap.comment));
                add_row("width", format!("{:?}", bitmap.width));
                add_row("height", format!("{:?}", bitmap.height));
                add_row("num_images", format!("{:?}", bitmap.num_images));
                add_row("start_index", format!("{:?}", bitmap.start_index));
                add_row("end_index", format!("{:?}", bitmap.end_index));
                add_row("image_width", format!("{:?}", bitmap.image_width));
                add_row("image_height", format!("{:?}", bitmap.image_height));
                add_row("file_size_555", format!("{:?}", bitmap.file_size_555));
                add_row("total_file_size", format!("{:?}", bitmap.total_file_size));
                add_row("file_size_external", format!("{:?}", bitmap.file_size_external));
                add_row("image_id", format!("{:?}", bitmap.image_id));
                add_row("unknown_a", format!("{:?}", bitmap.unknown_a));
                add_row("unknown_b", format!("{:?}", bitmap.unknown_b));
                add_row("unknown_c", format!("{:?}", bitmap.unknown_c));
                add_row("unknown_d", format!("{:?}", bitmap.unknown_d));
                add_row("unknown_e", format!("{:?}", bitmap.unknown_e));
                add_row("", format!(" "));
                add_row("filename", format!("{:?}", file.filename));
                add_row("file_size", format!("{:?}", file.file_size));
                add_row("version", format!("{:?}", file.version));
                add_row("unknown", format!("{:?}", file.unknown));
                add_row("max_image_count", format!("{:?}", file.max_image_count));
                add_row("bitmap_records_without_system", format!("{:?}", file.bitmap_records_without_system));
                add_row("total_file_size", format!("{:?}", file.total_file_size));
                add_row("file_size_555", format!("{:?}", file.file_size_555));
                add_row("file_size_external", format!("{:?}", file.file_size_external));

                return Scroll::new(Split::columns(labels, values).split_point(0.5).draggable(false)).vertical();
            }
            panic!("Image is selected, but no file is loaded!");
        }));
    });

    // Show selected image
    let image_widget = ViewSwitcher::new(|data: &AppData, _env| data.current_image, move |current_image, data: &AppData, _env| {
        return Box::new(current_image.map_or_else(|| {
            Label::new("No image selected").center()
        }, |image_id| {
            if let Some(LoadedFile(file)) = &data.loaded_file {
                let image = &file.images[image_id];
                let pixels: Vec<u8> = data.pixels.iter().cloned().collect();
                let format = ImageFormat::RgbaSeparate;
                let width = image.width;
                let height = image.height;

                let image_buf = ImageBuf::from_raw(pixels, format, width as usize, height as usize);
                let image_widget = Image::new(image_buf).border(Color::WHITE, 1.0).center();

                return image_widget.center();
            }
            panic!("Image is selected, but no file is loaded!");
        }));
    });

    let right = Split::columns(image_widget, table)
        .split_point(0.9)
        .bar_size(5.0)
        .draggable(true)
        .min_size(10.0, 300.0);

    return Container::new(
        Split::columns(left, right)
            .split_point(0.1)
            .bar_size(5.0)
            .draggable(true)
            .min_size(200.0, 200.0)
    );
}

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppData,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            match SgFileMetadata::load_metadata(file_info.path()) {
                Ok(sg_file) => {
                    data.images.clear();

                    for sg_image in &sg_file.images {
                        let label = format!("{} [{}x{}]", sg_image.id, sg_image.width, sg_image.height);
                        data.images.push_back((sg_image.id, label));
                    }

                    data.loaded_file = Option::Some(LoadedFile(sg_file));
                    data.title = String::from(file_info.path().as_os_str().to_str().unwrap());
                    data.current_image = Option::None;
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
            return Handled::Yes;
        }

        if let Some(image_id) = cmd.get(SELECT_IMAGE) {
            if let Some(LoadedFile(file)) = &data.loaded_file {
                let image = &file.images[*image_id as usize];
                let path = file.get_555_file_path(image.bitmap_id as usize, image.is_external());
                let mut reader = BufReader::new(File::open(path).expect("Failed to open file."));
                let pixels = image.load_image(&mut reader, &VecImageBuilderFactory).expect("Failed to get pixel data.");
                data.current_image = Option::Some(*image_id as usize);
                data.pixels = Vector::from(pixels);
                return Handled::Yes;
            }
        }

        Handled::No
    }
}

pub fn main() {
    fn title(app_data: &AppData, _env: &druid::Env) -> String {
        return app_data.title.clone();
    }

    let window = WindowDesc::new(build_app).title(title);

    AppLauncher::with_window(window)
        .delegate(Delegate)
        .use_simple_logger()
        .launch(AppData { title: String::from("SgViewerExample"), ..Default::default() })
        .expect("launch failed");
}

impl Lens<AppData, Option<LoadedFile>> for LoadedFile {
    fn with<V, F: FnOnce(&Option<LoadedFile>) -> V>(&self, data: &AppData, f: F) -> V {
        return f(&data.loaded_file);
    }

    fn with_mut<V, F: FnOnce(&mut Option<LoadedFile>) -> V>(&self, data: &mut AppData, f: F) -> V {
        return f(&mut data.loaded_file);
    }
}

impl Data for LoadedFile {
    fn same(&self, other: &Self) -> bool {
        return self.0.eq(&other.0);
    }
}