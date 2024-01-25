use std::{io::Cursor, path::PathBuf};

use base64::{engine::general_purpose, Engine};
use bevy::{
    prelude::*,
    render::{camera::RenderTarget, renderer::RenderDevice},
};
use image::{ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use super::image_copy::ImageCopier;

#[derive(Component, Default)]
pub struct CaptureCamera;

#[derive(Resource)]
pub struct CurrImageBase64(pub String);

#[derive(Component, Deref, DerefMut)]
pub struct ImageToSave(Handle<Image>);

#[derive(Debug, Default, Resource, Event)]
pub struct SceneInfo {
    state: SceneState,
    name: String,
    width: u32,
    height: u32,
}

impl SceneInfo {
    pub fn new(width: u32, height: u32) -> SceneInfo {
        SceneInfo { state: SceneState::BuildScene, name: String::from(""), width, height }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[derive(Debug, Default)]
pub enum SceneState {
    #[default]
    BuildScene,
    Render(u32),
}

impl SceneState {
    pub fn decrement(&mut self) {
        if let SceneState::Render(n) = self {
            *n -= 1;
        }
    }
}

pub fn setup_render_target(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    render_device: &Res<RenderDevice>,
    scene_controller: &mut ResMut<SceneInfo>,
    pre_roll_frames: u32,
    scene_name: String,
) -> RenderTarget {
    let size = Extent3d {
        width: scene_controller.width,
        height: scene_controller.height,
        ..Default::default()
    };

    // This is the texture that will be rendered to.
    let mut render_target_image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };
    render_target_image.resize(size);
    let render_target_image_handle = images.add(render_target_image);

    // This is the texture that will be copied to.
    let mut cpu_image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        ..Default::default()
    };
    cpu_image.resize(size);
    let cpu_image_handle = images.add(cpu_image);

    commands.spawn(ImageCopier::new(
        render_target_image_handle.clone(),
        cpu_image_handle.clone(),
        size,
        render_device,
    ));

    commands.spawn(ImageToSave(cpu_image_handle));

    scene_controller.state = SceneState::Render(pre_roll_frames);
    scene_controller.name = scene_name;
    RenderTarget::Image(render_target_image_handle)
}

pub fn update(
    images_to_save: Query<&ImageToSave>,
    mut images: ResMut<Assets<Image>>,
    mut curr_base64_img: Option<ResMut<CurrImageBase64>>,
    mut scene_controller: ResMut<SceneInfo>,
) {
    if let SceneState::Render(n) = scene_controller.state {
        if n < 1 {
            for image in images_to_save.iter() {
                let img_bytes = images.get_mut(image.id()).unwrap();

                let rgba_img = img_bytes.clone().try_into_dynamic().unwrap().to_rgba8();

                if let Some(base64_img) = curr_base64_img.as_mut(){
                    base64_img.0 = image_to_browser_base64(&rgba_img).unwrap();
                } else {
                    let images_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_images");
                    log::info!("Saving image to: {images_dir:?}");
                    
                    std::fs::create_dir_all(&images_dir).unwrap();
    
                    let uuid = bevy::utils::Uuid::new_v4();
                    let image_path = images_dir.join(format!("{uuid}.png"));
                    if let Err(e) = rgba_img.save(image_path) {
                        panic!("Failed to save image: {}", e);
                    };
                };

            }
        } else {
            scene_controller.state.decrement();
        }
    }
}

// useful utils
fn image_to_browser_base64(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> anyhow::Result<String> {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)?;
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    Ok(format!("data:image/png;base64,{}", res_base64))
}

pub fn white_img_placeholder(w: u32, h: u32) -> String {
    let img = RgbaImage::new(w, h);
    // img.iter_mut().for_each(|pixel| *pixel = 255);
    image_to_browser_base64(&img).unwrap()
}
