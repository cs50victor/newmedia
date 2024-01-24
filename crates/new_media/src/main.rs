#![feature(ascii_char, async_closure, slice_pattern)]
mod controls;
mod server;

use bevy_ws_server::WsPlugin;

use bevy::{
    app::ScheduleRunnerPlugin, core::Name, core_pipeline::tonemapping::Tonemapping, log::LogPlugin,
    prelude::*, render::renderer::RenderDevice,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_gaussian_splatting::{GaussianCloud, GaussianSplattingBundle, GaussianSplattingPlugin};

use server::{receive_message, start_ws};

#[derive(Resource)]
pub struct StreamingFrameData {
    pixel_size: u32,
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut gaussian_assets: ResMut<Assets<GaussianCloud>>,
    mut scene_controller: ResMut<bevy_frame_capture::scene::SceneController>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    // let remote_file = Some("https://huggingface.co/datasets/cs50victor/splats/resolve/main/train/point_cloud/iteration_7000/point_cloud.gcloud");
    // TODO: figure out how to load remote files later
    // let splat_file = "splats/bonsai/point_cloud/iteration_7000/point_cloud.gcloud";
    // log::info!("loading {}", splat_file);
    // let cloud = asset_server.load(splat_file.to_string());

    let cloud = gaussian_assets.add(GaussianCloud::test_model());

    let render_target = bevy_frame_capture::scene::setup_render_target(
        &mut commands,
        &mut images,
        &render_device,
        &mut scene_controller,
        15,
        String::from("main_scene"),
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
        .filter_module("bevy_ws_server", log::LevelFilter::Info)
        .filter_module("bevy_ws_server", log::LevelFilter::Debug)
        .init();

    let config = AppConfig { width: 1920, height: 1080 };

    App::new()
    .insert_resource(bevy_frame_capture::scene::SceneController::new(config.width, config.height))
    .insert_resource(bevy_frame_capture::scene::CurrImageBase64(bevy_frame_capture::scene::white_img_placeholder(config.width, config.height)))
    .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
    .add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                close_when_requested: false,
            }).disable::<LogPlugin>(),
        WsPlugin,
        bevy_frame_capture::image_copy::ImageCopyPlugin,
        bevy_frame_capture::scene::CaptureFramePlugin,
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
        PanOrbitCameraPlugin,
        GaussianSplattingPlugin,
    ))
    .init_resource::<bevy_frame_capture::scene::SceneController>()
    .add_event::<bevy_frame_capture::scene::SceneController>()
    .add_systems(Startup, (start_ws, setup_gaussian_cloud))
    .add_systems(Update, (
        move_camera,
        receive_message
    ))
    // .add_systems(OnEnter(AppState::Active), setup_gaussian_cloud)
    .run();
}

fn move_camera(mut camera: Query<&mut Transform, With<Camera>>) {
    for mut transform in camera.iter_mut() {
        transform.translation.x += 0.0005;
        transform.translation.y += 0.0005;
        transform.translation.z += 0.0005;
    }
}