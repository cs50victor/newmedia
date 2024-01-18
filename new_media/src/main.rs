#![feature(ascii_char, async_closure, slice_pattern)]
mod controls;
mod frame_capture;
mod llm;
mod server;
mod video;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use frame_capture::scene::SceneController;
use image::RgbaImage;
// use actix_web::{middleware, web::Data, App, HttpServer};
use log::info;

use bevy::{
    app::ScheduleRunnerPlugin, core::Name, core_pipeline::tonemapping::Tonemapping, log::LogPlugin,
    prelude::*, render::renderer::RenderDevice, time::common_conditions::on_timer, tasks::AsyncComputeTaskPool,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_gaussian_splatting::{GaussianCloud, GaussianSplattingBundle, GaussianSplattingPlugin};

use pollster::FutureExt;

use futures::StreamExt;

use log::{error, warn};
use serde::{Deserialize, Serialize};

use crate::{
    controls::WorldControlChannel, llm::LLMChannel, server::RoomData,
};

pub const OPENAI_ORG_ID: &str = "OPENAI_ORG_ID";

#[derive(Resource)]
pub struct AsyncRuntime {
    rt: std::sync::Arc<tokio::runtime::Runtime>,
}

impl FromWorld for AsyncRuntime {
    fn from_world(_world: &mut World) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        Self { rt: std::sync::Arc::new(rt) }
    }
}


#[derive(Resource)]
pub struct StreamingFrameData {
    pixel_size: u32,
}

#[derive(Serialize, Deserialize)]
struct RoomText {
    message: String,
    timestamp: i64,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Idle,
    Active,
}

#[derive(Default, Debug, PartialEq)]
pub enum AppStateServerResource {
    #[default]
    Init,
    Idle,
    Active,
}

#[derive(Default, Debug, PartialEq)]
struct ParticipantRoomName(String);

impl From<AppState> for AppStateServerResource {
    fn from(value: AppState) -> Self {
        match value {
            AppState::Idle => AppStateServerResource::Idle,
            AppState::Active => AppStateServerResource::Active,
        }
    }
}

#[derive(Resource)]
pub struct AudioSync {
    should_stop: Arc<AtomicBool>,
}

#[derive(Resource)]
pub struct AppStateSync {
    state: std::sync::Arc<parking_lot::Mutex<ParticipantRoomName>>,
    dirty: bool,
}

fn setup_gaussian_cloud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    _gaussian_assets: ResMut<Assets<GaussianCloud>>,
    mut scene_controller: ResMut<frame_capture::scene::SceneController>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    // let remote_file = Some("https://huggingface.co/datasets/cs50victor/splats/resolve/main/train/point_cloud/iteration_7000/point_cloud.gcloud");
    // TODO: figure out how to load remote files later
    let splat_file = "splats/bonsai/point_cloud/iteration_7000/point_cloud.gcloud";
    log::info!("loading {}", splat_file);
    let cloud = asset_server.load(splat_file.to_string());

    // let cloud = gaussian_assets.add(GaussianCloud::test_model());

    let render_target = frame_capture::scene::setup_render_target(
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
            transform: Transform { translation: Vec3::new(-0.59989005, -0.88360703, -2.0863006), rotation: Quat::from_xyzw(-0.97177905, -0.026801618, 0.13693734, -0.1901983), scale: Vec3::new(1.0, 1.0, 1.0) },
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

pub fn sync_bevy_and_server_resources(
    mut commands: Commands,
    async_runtime: Res<AsyncRuntime>,
    mut server_state_clone: ResMut<AppStateSync>,
    mut set_app_state: ResMut<NextState<AppState>>,
    scene_controller: Res<SceneController>,
    audio_syncer: Res<AudioSync>,
) {
    if !server_state_clone.dirty {
        let participant_room_name = &(server_state_clone.state.lock().0).clone();
        if !participant_room_name.is_empty() {
            let video_frame_dimensions = scene_controller.dimensions();
            
        };
    }
}

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
}

fn main() {
    dotenvy::from_filename_override(".env.local").ok();

    // ************** REQUIRED ENV VARS **************
    std::env::var(OPENAI_ORG_ID).expect("OPENAI_ORG_ID must be set");

    let mut formatted_builder = pretty_env_logger::formatted_builder();

    let pretty_env_builder = formatted_builder
        .filter_module("lkgpt", log::LevelFilter::Info)
        .filter_module("actix_server", log::LevelFilter::Info)
        .filter_module("bevy", log::LevelFilter::Info)
        .filter_module("actix_web", log::LevelFilter::Info);

    if cfg!(target_os = "unix") {
        pretty_env_builder.filter_module("livekit", log::LevelFilter::Info);
    }

    pretty_env_builder.init();

    let mut app = App::new();

    let config = AppConfig { width: 1920, height: 1080 };

    app.insert_resource(frame_capture::scene::SceneController::new(config.width, config.height));
    app.insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)));

    app.add_plugins((
        bevy_web_asset::WebAssetPlugin,
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            // "headless" window
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                close_when_requested: false,
            }).disable::<LogPlugin>(),
        frame_capture::image_copy::ImageCopyPlugin,
        frame_capture::scene::CaptureFramePlugin,
        ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
        PanOrbitCameraPlugin,
        // plugin for gaussian splatting
        GaussianSplattingPlugin,
    ));

    app.add_state::<AppState>();
    app.init_resource::<AsyncRuntime>();
    app.insert_resource(AudioSync { should_stop: Arc::new(AtomicBool::new(false)) });
    app.init_resource::<server::ActixServer>();

    app.init_resource::<frame_capture::scene::SceneController>();
    app.add_event::<frame_capture::scene::SceneController>();

    app.add_systems(Update, move_camera);

    app.add_systems(Update, server::shutdown_bevy_remotely);

    // app.add_systems(
    //     Update,
    //     room_events::handle_room_events
    //         .run_if(resource_exists::<llm::LLMChannel>())
    //         .run_if(resource_exists::<stt::STT>())
    //         .run_if(resource_exists::<video::VideoChannel>())
    //         .run_if(resource_exists::<LivekitRoom>()),
    // );

    app.add_systems(
        Update,
        llm::run_llm
            .run_if(resource_exists::<llm::LLMChannel>())
            .run_if(in_state(AppState::Active)),
    );

    app.add_systems(
        Update,
        sync_bevy_and_server_resources.run_if(on_timer(std::time::Duration::from_secs(2))),
    );

    app.add_systems(OnEnter(AppState::Active), setup_gaussian_cloud);

    app.run();
}

fn move_camera(mut camera: Query<&mut Transform, With<Camera>>) {
    for mut transform in camera.iter_mut() {
        transform.translation.x += 0.0005;
        transform.translation.y += 0.0005;
        transform.translation.z += 0.0005;
    }
}
