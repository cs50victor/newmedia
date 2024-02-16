mod asset;
mod controls;
mod server;

use asset::setup_gaussian_cloud;
use bevy::{
    app::{App as Engine, ScheduleRunnerPlugin, Startup, Update},
    core_pipeline::clear_color::ClearColor,
    render::color::Color,
};
use bevy_headless::HeadlessPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

use bevy_gaussian_splatting::GaussianSplattingPlugin;
use bevy_remote_asset::WebAssetPlugin;
use bevy_ws_server::WsPlugin;
use controls::{update_world_from_input, WorldControlChannel};
use server::{receive_message, start_ws};

pub struct AppConfig {
    pub width: u32,
    pub height: u32,
}

fn main() {
    dotenvy::from_filename_override(".env.local").ok();

    pretty_env_logger::formatted_builder()
        .filter_module("new_media", log::LevelFilter::Info)
        .filter_module("bevy", log::LevelFilter::Info)
        .filter_module("bevy_headless", log::LevelFilter::Info)
        .filter_module("bevy_ws_server", log::LevelFilter::Info)
        .filter_module("bevy_ws_server", log::LevelFilter::Debug)
        .init();

    // TODO: change this a preset of resoltions with their aspect ratios
    let config = AppConfig { width: 1920, height: 1080 };

    Engine::new()
        .insert_resource(bevy_headless::SceneInfo::new(config.width, config.height))
        .init_resource::<WorldControlChannel>()
        // .insert_resource(ClearColor(Color::rgb_u8(255, 255, 255)))
        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugins((
            WebAssetPlugin,
            HeadlessPlugin,
            WsPlugin,
            ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
            PanOrbitCameraPlugin,
            GaussianSplattingPlugin,
        ))
        .add_systems(Startup, (start_ws, setup_gaussian_cloud))
        .add_systems(Update, (receive_message,update_world_from_input))
        .run();
}
