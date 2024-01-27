mod controls;
mod server;

use bevy::{
    app::{App as Engine, ScheduleRunnerPlugin, Startup, Update},
    asset::{AssetServer, Assets},
    core::Name,
    core_pipeline::{clear_color::ClearColor, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::{
        system::{Commands,  Res, ResMut},
    },
    math::Vec3,
    render::{camera::Camera, color::Color, texture::Image},
    transform::components::Transform,
    utils::default,
};
use bevy_headless::{HeadlessPlugin, ImageExportSource};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_gaussian_splatting::{GaussianCloud, GaussianSplattingBundle, GaussianSplattingPlugin};
use bevy_ws_server::WsPlugin;
use server::{receive_message, start_ws};

fn setup_gaussian_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _gaussian_assets: ResMut<Assets<GaussianCloud>>,
    mut scene_controller: ResMut<bevy_headless::SceneInfo>,
    mut images: ResMut<Assets<Image>>,
    export_sources: ResMut<Assets<ImageExportSource>>,
) {
    // let remote_file = Some("https://huggingface.co/datasets/cs50victor/splats/resolve/main/train/point_cloud/iteration_7000/point_cloud.gcloud");
    // TODO: figure out how to load remote files later
    let splat_file = "splats/bonsai/point_cloud/iteration_7000/point_cloud.gcloud";
    log::info!("loading {}", splat_file);
    let cloud = asset_server.load(splat_file.to_string());

    // let cloud = gaussian_assets.add(GaussianCloud::test_model());

    let render_target = bevy_headless::setup_render_target(
        &mut commands,
        &mut images,
        &mut scene_controller,
        export_sources,
    );

    let gs = GaussianSplattingBundle { cloud, ..default() };
    commands.spawn((gs, Name::new("gaussian_cloud")));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            tonemapping: Tonemapping::None,
            camera: Camera { target: render_target, ..default() },
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

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
}

fn main() {
    dotenvy::from_filename_override(".env.local").ok();
    // ************** REQUIRED ENV VARS **************

    pretty_env_logger::formatted_builder()
        .filter_module("new_media", log::LevelFilter::Info)
        .filter_module("bevy", log::LevelFilter::Info)
        .filter_module("bevy_headless", log::LevelFilter::Info)
        .filter_module("bevy_ws_server", log::LevelFilter::Info)
        .filter_module("bevy_ws_server", log::LevelFilter::Debug)
        .init();

    let config = AppConfig { width: 1920, height: 1080 };

    Engine::new()
        .insert_resource(bevy_headless::SceneInfo::new(config.width, config.height))
        .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .add_plugins((
            HeadlessPlugin,
            WsPlugin,
            ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
            PanOrbitCameraPlugin,
            GaussianSplattingPlugin,
        ))
        .add_systems(Startup, (start_ws, setup_gaussian_cloud))
        .add_systems(Update, receive_message)
        .run();
}
