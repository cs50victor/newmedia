// use std::io::Cursor;

// use anyhow::Result;
// use base64::{engine::general_purpose, Engine};
// use bevy::{
//     prelude::*,
//     render::{camera::RenderTarget, renderer::RenderDevice},
// };

// use image::{ImageBuffer, ImageOutputFormat, Rgba, RgbaImage};
// use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

// use super::image_copy::ImageCopier;

// #[derive(Component, Default)]
// pub struct CaptureCamera;

// #[derive(Component, Deref, DerefMut)]
// struct ImageToSave(Handle<Image>);

// #[derive(Resource)]
// pub struct CurrImageBase64(pub String);

// #[derive(Resource)]
// pub struct StreamingFrameData {
//     pixel_size: u32,
// }

// pub struct CaptureFramePlugin;
// impl Plugin for CaptureFramePlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(PostUpdate, update.run_if(resource_exists::<StreamingFrameData>()));
//     }
// }

// #[derive(Debug, Default, Resource, Event)]
// pub struct SceneController {
//     state: SceneState,
//     name: String,
//     width: u32,
//     height: u32,
// }

// impl SceneController {
//     pub fn new(width: u32, height: u32) -> SceneController {
//         SceneController { state: SceneState::BuildScene, name: String::from(""), width, height }
//     }

//     pub fn dimensions(&self) -> (u32, u32) {
//         (self.width, self.height)
//     }
// }

// #[derive(Debug, Default)]
// pub enum SceneState {
//     #[default]
//     BuildScene,
//     Render(u32),
// }

// impl SceneState {
//     pub fn decrement(&mut self) {
//         if let SceneState::Render(n) = self {
//             *n -= 1;
//         }
//     }
// }

// pub fn setup_render_target(
//     commands: &mut Commands,
//     images: &mut ResMut<Assets<Image>>,
//     render_device: &Res<RenderDevice>,
//     scene_controller: &mut ResMut<SceneController>,
//     pre_roll_frames: u32,
//     scene_name: String,
// ) -> RenderTarget {
//     let size = Extent3d {
//         width: scene_controller.width,
//         height: scene_controller.height,
//         ..Default::default()
//     };

//     // This is the texture that will be rendered to.
//     let mut render_target_image = Image {
//         texture_descriptor: TextureDescriptor {
//             label: None,
//             size,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Rgba8UnormSrgb,
//             mip_level_count: 1,
//             sample_count: 1,
//             usage: TextureUsages::COPY_SRC
//                 | TextureUsages::COPY_DST
//                 | TextureUsages::TEXTURE_BINDING
//                 | TextureUsages::RENDER_ATTACHMENT,
//             view_formats: &[],
//         },
//         ..Default::default()
//     };
//     render_target_image.resize(size);
//     let render_target_image_handle = images.add(render_target_image);

//     // This is the texture that will be copied to.
//     let mut cpu_image = Image {
//         texture_descriptor: TextureDescriptor {
//             label: None,
//             size,
//             dimension: TextureDimension::D2,
//             format: TextureFormat::Rgba8UnormSrgb,
//             mip_level_count: 1,
//             sample_count: 1,
//             usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
//             view_formats: &[],
//         },
//         ..Default::default()
//     };
//     cpu_image.resize(size);
//     let cpu_image_handle = images.add(cpu_image);

//     commands.spawn(ImageCopier::new(
//         render_target_image_handle.clone(),
//         cpu_image_handle.clone(),
//         size,
//         render_device,
//     ));

//     commands.spawn(ImageToSave(cpu_image_handle));

//     scene_controller.state = SceneState::Render(pre_roll_frames);
//     scene_controller.name = scene_name;
//     RenderTarget::Image(render_target_image_handle)
// }

// fn update(
//     mut images: ResMut<Assets<Image>>,
//     images_to_save: Query<&ImageToSave>,
//     mut curr_base64_img: ResMut<CurrImageBase64>,
//     single_frame_data: ResMut<StreamingFrameData>,
//     mut scene_controller: ResMut<SceneController>,
// ) {
//     if let SceneState::Render(n) = scene_controller.state {
//         if n < 1 {
//             let single_frame_data = single_frame_data.into_inner();
//             let _pixel_size = single_frame_data.pixel_size;
//             for image in images_to_save.iter() {
//                 let img_bytes = images.get_mut(image.id()).unwrap();

//                 let rgba_img = match img_bytes.clone().try_into_dynamic() {
//                     Ok(img) => img.to_rgba8(),
//                     Err(e) => panic!("Failed to create image buffer {e:?}"),
//                 };

//                 println!("wtf");
//                 rgba_img.save("wtf.png");
//                 log::info!("saved");
//                 curr_base64_img.0 = image_to_browser_base64(&rgba_img).unwrap();
//             }
//         } else {
//             scene_controller.state.decrement();
//         }
//     }
// }

// fn image_to_browser_base64(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String> {
//     let mut image_data: Vec<u8> = Vec::new();
//     img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)?;
//     let res_base64 = general_purpose::STANDARD.encode(image_data);
//     Ok(format!("data:image/png;base64,{}", res_base64))
// }

// pub fn white_img_placeholder(w: u32, h: u32) -> String {
//     let img = RgbaImage::new(w, h);
//     // img.iter_mut().for_each(|pixel| *pixel = 255);
//     image_to_browser_base64(&img).unwrap()
// }




    use std::path::PathBuf;

    use bevy::{
        app::AppExit,
        prelude::*,
        render::{camera::RenderTarget, renderer::RenderDevice},
    };
    use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

    use super::image_copy::ImageCopier;

    #[derive(Component, Default)]
    pub struct CaptureCamera;

    #[derive(Component, Deref, DerefMut)]
    struct ImageToSave(Handle<Image>);

    pub struct CaptureFramePlugin;
    impl Plugin for CaptureFramePlugin {
        fn build(&self, app: &mut App) {
            println!("Adding CaptureFramePlugin");
            app.add_systems(PostUpdate, update);
        }
    }

    #[derive(Debug, Default, Resource, Event)]
    pub struct SceneController {
        state: SceneState,
        name: String,
        width: u32,
        height: u32,
        single_image: bool,
    }

    impl SceneController {
        pub fn new(width: u32, height: u32, single_image: bool) -> SceneController {
            SceneController {
                state: SceneState::BuildScene,
                name: String::from(""),
                width,
                height,
                single_image,
            }
        }
    }

    #[derive(Debug, Default)]
    pub enum SceneState {
        #[default]
        BuildScene,
        Render(u32),
    }

    pub fn setup_render_target(
        commands: &mut Commands,
        images: &mut ResMut<Assets<Image>>,
        render_device: &Res<RenderDevice>,
        scene_controller: &mut ResMut<SceneController>,
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

    fn update(
        images_to_save: Query<&ImageToSave>,
        mut images: ResMut<Assets<Image>>,
        mut scene_controller: ResMut<SceneController>,
        mut app_exit_writer: EventWriter<AppExit>,
    ) {
        if let SceneState::Render(n) = scene_controller.state {
            if n < 1 {
                for image in images_to_save.iter() {
                    let img_bytes = images.get_mut(image.id()).unwrap();

                    let img = match img_bytes.clone().try_into_dynamic() {
                        Ok(img) => img.to_rgba8(),
                        Err(e) => panic!("Failed to create image buffer {e:?}"),
                    };

                    let images_dir =
                        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_images");
                    print!("Saving image to: {:?}\n", images_dir);
                    std::fs::create_dir_all(&images_dir).unwrap();

                    let uuid = bevy::utils::Uuid::new_v4();
                    let image_path = images_dir.join(format!("{uuid}.png"));
                    if let Err(e) = img.save(image_path) {
                        panic!("Failed to save image: {}", e);
                    };
                }
            } else {
                scene_controller.state = SceneState::Render(n - 1);
            }
        }
    }
