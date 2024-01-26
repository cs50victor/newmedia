use bevy::{
    asset::Assets,
    ecs::{
        event::Event,
        system::{Commands, ResMut, Resource},
    },
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::Image,
    },
};
use std::{io::Cursor, ops::Deref};

use base64::{engine::general_purpose, Engine};
use image::{EncodableLayout, ImageBuffer, ImageOutputFormat, Pixel, Rgba, RgbaImage};

use crate::{ImageExportBundle, ImageExportSource};

#[derive(Default, Resource)]
pub struct CurrImage {
    pub img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub frame_id: u64,
    pub extension: String,
}

impl CurrImage {
    pub fn update_data<P, Container>(
        &mut self,
        frame_id: u64,
        image_bytes: &ImageBuffer<P, Container>,
        extension: String,
    ) where
        P: Pixel + image::PixelWithColorType,
        [P::Subpixel]: EncodableLayout,
        Container: Deref<Target = [P::Subpixel]>,
    {
        self.frame_id = frame_id;

        self.extension = extension;

        let (w, h) = image_bytes.dimensions();
        if let Some(rgba_img_buff) = RgbaImage::from_raw(w, h, image_bytes.as_bytes().to_owned()) {
            self.img_buffer = rgba_img_buff;
        } else {
            log::error!("Error updating curr image image buffer");
        };
    }

    pub fn create_path(&self, dir: &str) -> String {
        // shouldn't be in loop, remove later
        std::fs::create_dir_all(dir).expect("Output path could not be created");

        format!("{dir}/{:06}.{}", self.frame_id, self.extension)
    }

    pub fn to_web_base64(&self) -> anyhow::Result<String> {
        base64_browser_img(&self.img_buffer)
    }
}

#[derive(Debug, Default, Resource, Event)]
pub struct SceneInfo {
    width: u32,
    height: u32,
}

impl SceneInfo {
    pub fn new(width: u32, height: u32) -> SceneInfo {
        SceneInfo { width, height }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub fn setup_render_target(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    scene_controller: &mut ResMut<SceneInfo>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
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
                // ?? remove ??
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };
    render_target_image.resize(size);
    let render_target_image_handle = images.add(render_target_image);

    commands.spawn(ImageExportBundle {
        source: export_sources.add(render_target_image_handle.clone().into()),
        ..Default::default()
    });

    RenderTarget::Image(render_target_image_handle)
}

fn base64_browser_img<P, Container>(img: &ImageBuffer<P, Container>) -> anyhow::Result<String>
where
    P: Pixel + image::PixelWithColorType,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::WebP)?;
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    Ok(format!("data:image/webp;base64,{}", res_base64))
}

fn white_img_placeholder(w: u32, h: u32) -> String {
    let img = RgbaImage::new(w, h);

    // img.iter_mut().for_each(|pixel| *pixel = 255);
    base64_browser_img(&img).unwrap()
}
