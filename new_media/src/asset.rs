use bevy::{
    asset::{AssetServer, Assets},
    core::Name,
    core_pipeline::{core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::system::{Commands, Res, ResMut},
    math::Vec3,
    render::{camera::Camera, texture::Image},
    transform::components::Transform,
    utils::default,
};
use bevy_headless::ImageExportSource;
use bevy_panorbit_camera::PanOrbitCamera;

use bevy_gaussian_splatting::{GaussianCloud, GaussianSplattingBundle};

pub fn setup_gaussian_cloud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cloud = asset_server.load::<GaussianCloud>("https://huggingface.co/datasets/cs50victor/splats/resolve/main/train/point_cloud/iteration_7000/point_cloud.gcloud");

    log::info!("loading {}", cloud.id());

    let gs = GaussianSplattingBundle { cloud, ..default() };
    commands.spawn((gs, Name::new("gaussian_cloud")));
}

pub fn setup_camera(
    mut commands: Commands,
    mut scene_controller: ResMut<bevy_headless::SceneInfo>,
    mut images: ResMut<Assets<Image>>,
    export_sources: ResMut<Assets<ImageExportSource>>,
) {
    let target = bevy_headless::setup_render_target(
        &mut commands,
        &mut images,
        &mut scene_controller,
        export_sources,
    );

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            tonemapping: Tonemapping::None,
            camera: Camera { target, ..default() },
            ..default()
        },
        PanOrbitCamera {
            allow_upside_down: true,
            orbit_smoothness: 0.0,
            pan_smoothness: 0.0,
            zoom_smoothness: 0.0,
            ..default()
        },
    ));
}
