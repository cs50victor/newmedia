use std::sync::Arc;

use crate::{
    node::{ImageExportNode, NODE_NAME},
    utils::{CurrImage},
};
use bevy::{
    app::{App, Plugin, PluginGroup, PostUpdate},
    asset::{Asset, AssetApp, Handle},
    ecs::{
        bundle::Bundle, component::Component, entity::Entity, query::{QueryItem, With, Without}, schedule::{apply_deferred, IntoSystemConfigs, IntoSystemSetConfigs, SystemSet}, system::{lifetimeless::SRes, Commands, Local, Query, Res, ResMut, Resource, SystemParamItem}, world::Mut
    },
    log::LogPlugin,
    reflect::{Reflect, TypeUuid},
    render::{
        camera::CameraUpdateSystem,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_graph::RenderGraph,
        render_resource::{Buffer, BufferDescriptor, BufferUsages, Extent3d, MapMode},
        renderer::RenderDevice,
        texture::{Image, ImagePlugin},
        Render, RenderApp, RenderSet,
    },
    window::WindowPlugin,
    DefaultPlugins,
};
use bytemuck::AnyBitPattern;
use futures::channel::oneshot;
use image::{EncodableLayout, ImageBuffer, Pixel, PixelWithColorType, Rgba};

use parking_lot::Mutex;
use wgpu::Maintain;
use ImageExportSystems::{SetupImageExport, SetupImageExportFlush};

#[derive(Asset, Clone, TypeUuid, Default, Reflect)]
#[uuid = "d619b2f8-58cf-42f6-b7da-028c0595f7aa"]
pub struct ImageExportSource(pub Handle<Image>);

impl From<Handle<Image>> for ImageExportSource {
    fn from(value: Handle<Image>) -> Self {
        Self(value)
    }
}

#[derive(Component, Clone)]
pub struct ImageExportSettings {
    /// The image file extension. "png", "jpeg", "webp", or "exr".
    pub extension: String,
}

pub struct GpuImageExportSource {
    pub buffer: Buffer,
    pub source_handle: Handle<Image>,
    pub source_size: Extent3d,
    pub bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
}

impl GpuImageExportSource {
    fn get_bps(&self) -> (usize, usize, Extent3d) {
        (self.bytes_per_row as usize, self.padded_bytes_per_row as usize, self.source_size)
    }
}

impl RenderAsset for ImageExportSource {
    type ExtractedAsset = Self;
    type Param = (SRes<RenderDevice>, SRes<RenderAssets<Image>>);
    type PreparedAsset = GpuImageExportSource;

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (device, images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let gpu_image = images.get(&extracted_asset.0).unwrap();

        let size = gpu_image.texture.size();
        let format = &gpu_image.texture_format;
        let bytes_per_row =
            (size.width / format.block_dimensions().0) * format.block_size(None).unwrap();
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

        let source_size = gpu_image.texture.size();

        Ok(GpuImageExportSource {
            buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Image Export Buffer"),
                size: (source_size.height * padded_bytes_per_row) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            source_handle: extracted_asset.0.clone(),
            source_size,
            bytes_per_row,
            padded_bytes_per_row,
        })
    }
}

#[derive(Component, Clone)]
pub struct ImageExportStartFrame(u64);

impl Default for ImageExportSettings {
    fn default() -> Self {
        Self { extension: "png".into() }
    }
}

impl ExtractComponent for ImageExportSettings {
    type Filter = ();
    type Out = (Self, Handle<ImageExportSource>, ImageExportStartFrame);
    type Query =
        (&'static Self, &'static Handle<ImageExportSource>, &'static ImageExportStartFrame);

    fn extract_component(
        (settings, source_handle, start_frame): QueryItem<'_, Self::Query>,
    ) -> Option<Self::Out> {
        Some((settings.clone(), source_handle.clone_weak(), start_frame.clone()))
    }
}

fn setup_exporters(
    mut commands: Commands,
    exporters: Query<Entity, (With<ImageExportSettings>, Without<ImageExportStartFrame>)>,
    mut frame_id: Local<u64>,
) {
    *frame_id = frame_id.wrapping_add(1);
    for entity in &exporters {
        commands.entity(entity).insert(ImageExportStartFrame(*frame_id));
    }
}

#[derive(Bundle, Default)]
pub struct ImageExportBundle {
    pub source: Handle<ImageExportSource>,
    pub settings: ImageExportSettings,
}

fn save_buffer_as_resource(
    export_bundles: Query<(
        &Handle<ImageExportSource>,
        &ImageExportSettings,
        &ImageExportStartFrame,
    )>,
    sources: Res<RenderAssets<ImageExportSource>>,
    render_device: Res<RenderDevice>,
    mut curr_img: ResMut<CurrImageContainer>,
    mut frame_id: Local<u64>,
) {
    *frame_id = frame_id.wrapping_add(1);

    log::debug!("num of export bundles {}", export_bundles.iter().len());

    for (source_handle, settings, start_frame) in &export_bundles {
        if let Some(gpu_source) = sources.get(source_handle) {
            let mut image_bytes = {
                let slice = gpu_source.buffer.slice(..);

                {
                    let (mapping_tx, mapping_rx) = oneshot::channel();

                    render_device.map_buffer(&slice, MapMode::Read, move |res| {
                        mapping_tx.send(res).unwrap();
                    });

                    render_device.poll(Maintain::Wait);
                    futures_lite::future::block_on(mapping_rx).unwrap().unwrap();
                }

                slice.get_mapped_range().to_vec()
            };

            gpu_source.buffer.unmap();

            let settings = settings.clone();
            let frame_id = *frame_id - start_frame.0 + 1;
            let (bytes_per_row, padded_bytes_per_row, source_size) = gpu_source.get_bps();

            if bytes_per_row != padded_bytes_per_row {
                let mut unpadded_bytes =
                    Vec::<u8>::with_capacity(source_size.height as usize * bytes_per_row);

                for padded_row in image_bytes.chunks(padded_bytes_per_row) {
                    unpadded_bytes.extend_from_slice(&padded_row[..bytes_per_row]);
                }

                image_bytes = unpadded_bytes;
            }

            let extension = settings.extension.as_str();
            match extension {
                "exr" => {
                    capture_img_bytes::<Rgba<f32>>(
                        bytemuck::cast_slice(&image_bytes),
                        &source_size,
                        &mut curr_img,
                        frame_id,
                        extension,
                    );
                },
                _ => {
                    capture_img_bytes::<Rgba<u8>>(
                        &image_bytes,
                        &source_size,
                        &mut curr_img,
                        frame_id,
                        extension,
                    );
                },
            }
        }
    }
}

fn capture_img_bytes<P: Pixel + PixelWithColorType>(
    image_bytes: &[P::Subpixel],
    source_size: &Extent3d,
    curr_img: &mut ResMut<CurrImageContainer>,
    frame_id: u64,
    extension: &str,
) where
    P::Subpixel: AnyBitPattern,
    [P::Subpixel]: EncodableLayout,
{
    match ImageBuffer::<P, _>::from_raw(source_size.width, source_size.height, image_bytes) {
        Some(image_bytes) => {
            curr_img.0.lock().update_data(frame_id, &image_bytes, extension.to_owned());

            // if let Some(curr_img) = curr_img.as_mut(){
            //     curr_img.update_data(frame_id, &image_bytes, extension.to_owned());
            // }else{
            //     log::info!("Curr Image Resource still doesn't exist");
            // }

        },
        None => {
            log::error!("Failed creating image buffer for frame - '{frame_id}'");
        },
    }
}

/// Plugin enabling the generation of image sequences.
#[derive(Default)]
pub struct ImageExportPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ImageExportSystems {
    SetupImageExport,
    SetupImageExportFlush,
}

impl Plugin for ImageExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    close_when_requested: false,
                })
                .disable::<LogPlugin>(),
        );

        // TODO: 
        let curr_image_container = CurrImageContainer::default();

        app.insert_resource(curr_image_container.clone());
        
        app.configure_sets(
            PostUpdate,
            (SetupImageExport, SetupImageExportFlush).chain().before(CameraUpdateSystem),
        )
        .register_type::<ImageExportSource>()
        .init_asset::<ImageExportSource>()
        .register_asset_reflect::<ImageExportSource>()
        .add_plugins((
            RenderAssetPlugin::<ImageExportSource>::default(),
            ExtractComponentPlugin::<ImageExportSettings>::default(),
        ))
        .add_systems(
            PostUpdate,
            (
                setup_exporters.in_set(SetupImageExport),
                apply_deferred.in_set(SetupImageExportFlush),
            ),
        );

        let render_app = app.sub_app_mut(RenderApp);
        
        render_app.insert_resource(curr_image_container);

        render_app.add_systems(
            Render,
            save_buffer_as_resource.after(RenderSet::Render).before(RenderSet::Cleanup),
        );

        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        graph.add_node(NODE_NAME, ImageExportNode);
        graph.add_node_edge(CAMERA_DRIVER, NODE_NAME);
    

    }
}

#[derive(Clone, Default, Resource)]
pub struct CurrImageContainer(pub Arc<Mutex<CurrImage>>);