extern crate druid;
extern crate piet_common;

use std::vec::Vec;

use druid::*;
use druid::im::Vector;
use druid::widget::{Button, Container, Flex, Image, List, Scroll, Spinner, Split, ViewSwitcher};

use sg_reader::{SgFileMetadata, VecImageBuilderFactory};

use crate::piet::ImageFormat;

#[derive(Clone, Data, Lens)]
struct LoadedImage {
    id: usize,
    data: Vector<u8>,
    width: u16,
    height: u16,
    group: u8,
}

#[derive(Clone, Data, Lens)]
struct LoadedGroup {
    id: u8,
    label: String,
}

#[derive(Clone, Data, Lens)]
struct AppData {
    title: String,
    images: Vector<LoadedImage>,
    group_images: Vector<LoadedImage>,
    groups: Vector<LoadedGroup>,
    current_image: Option<usize>,
}

struct Delegate;

pub const SELECT_GROUP: Selector<u8> = Selector::new("select-sg-group");
pub const SELECT_IMAGE: Selector<usize> = Selector::new("select-sg-image");

fn build_app() -> impl Widget<AppData> {
    let groups = Scroll::new(List::new(|| {
        Button::new(|item: &LoadedGroup, _env: &_| item.label.clone())
            .on_click(|ctx, data, _| {
                ctx.submit_command(Command::new(
                    SELECT_GROUP,
                    data.id,
                    Target::Auto,
                ))
            })
            .expand_width()
    })).vertical()
        .lens(AppData::groups)
        .expand();

    let images = Scroll::new(List::new(|| {
        Button::new(|item: &LoadedImage, _env: &_| format!("{} [{}x{}]", item.id, item.width, item.height))
            .on_click(|ctx, data, _| {
                ctx.submit_command(Command::new(
                    SELECT_IMAGE,
                    data.id,
                    Target::Auto,
                ))
            })
            .expand_width()
    })).vertical()
        .lens(AppData::group_images)
        .expand();


    // Create the button to open a filk
    let open_dialog_options = FileDialogOptions::new()
        .allowed_types(vec![FileSpec::new("SG3", &["sg3"])]);

    let mut left = Flex::column();

    left.add_child(
        Button::new("Open file").on_click(move |ctx, _, _| {
            ctx.submit_command(Command::new(
                druid::commands::SHOW_OPEN_PANEL,
                open_dialog_options.clone(),
                Target::Auto,
            ))
        }).expand_width()
    );

    left.add_flex_child(
        Split::rows(groups, images)
            .split_point(0.5)
            .bar_size(5.0)
            .draggable(true)
            .min_size(200.0, 200.0),
        1.0,
    );


    // Show selected image
    let right = ViewSwitcher::new(|data: &AppData, _env| data.current_image, move |current_image, data: &AppData, _env| {
        return Box::new(current_image.map_or_else(|| {
            Spinner::new().fix_height(60.0).center()
        }, |img_id| {
            let image = &data.images[img_id];
            let pixels: Vec<u8> = image.data.iter().cloned().collect();
            let format = ImageFormat::RgbaSeparate;
            let width = image.width;
            let height = image.height;

            let image_buf = ImageBuf::from_raw(pixels, format, width as usize, height as usize);
            Image::new(image_buf).border(Color::WHITE, 1.0).center()
        }));
    });

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
            match SgFileMetadata::load_fully(file_info.path(), &VecImageBuilderFactory) {
                Ok((sg_file, images)) => {
                    data.groups.clear();
                    data.group_images.clear();
                    data.images.clear();

                    for (sg_image, pixels) in sg_file.images.iter().zip(images) {
                        data.images.push_back(LoadedImage {
                            id: sg_image.id as usize,
                            data: Vector::from(pixels),
                            height: sg_image.height,
                            width: sg_image.width,
                            group: sg_image.bitmap_id,
                        });
                    }

                    for (id, group) in sg_file.bitmaps.iter().enumerate() {
                        let label = if group.comment.is_empty() {
                            format!("{}", group.external_filename)
                        } else {
                            format!("{} ({})", group.external_filename, group.comment)
                        };
                        data.groups.push_back(LoadedGroup { id: id as u8, label });
                    }

                    data.title = String::from(file_info.path().as_os_str().to_str().unwrap());
                }
                Err(e) => {
                    println!("Error opening file: {}", e);
                }
            }
            return Handled::Yes;
        }

        if let Some(image_id) = cmd.get(SELECT_IMAGE) {
            data.current_image = Option::Some(*image_id);
            return Handled::Yes;
        }

        if let Some(group_id) = cmd.get(SELECT_GROUP) {
            data.group_images = data.images.iter().filter(|img| img.group == *group_id).cloned().collect();
            return Handled::Yes;
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
        .launch(AppData {
            title: String::from("SgViewerExaple"),
            images: Vector::new(),
            group_images: Vector::new(),
            groups: Vector::new(),
            current_image: Option::None,
        })
        .expect("launch failed");
}
