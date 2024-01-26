use bevy::{
    app::{App as Engine, ScheduleRunnerPlugin, Startup, Update},
    asset::Assets,
    core_pipeline::{clear_color::ClearColor, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::system::{Commands, Res, ResMut},
    math::Vec3,
    render::{camera::Camera, color::Color, texture::Image},
    transform::components::Transform,
    utils::default,
};
use bevy_headless::{CurrImageContainer, ImageExportPlugin, ImageExportSource};

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<bevy_headless::SceneInfo>,
    export_sources: ResMut<Assets<ImageExportSource>>,
) {
    let render_target = bevy_headless::setup_render_target(
        &mut commands,
        &mut images,
        &mut scene_controller,
        export_sources,
    );

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        tonemapping: Tonemapping::None,
        camera: Camera { target: render_target, ..default() },
        ..default()
    });
}

fn save_img(curr_img: Res<CurrImageContainer>) {
    let curr_img = curr_img.0.lock();
    if !curr_img.extension.is_empty() {
        let path = curr_img.create_path("out");
        log::info!("path is {path}");
        let img = curr_img.img_buffer.clone();
        std::thread::spawn(move || {
            if let Err(e) = img.save(path) {
                log::error!("Couldn't save image | {e:?}");
            };
        });
    }
}

pub fn main() {
    pretty_env_logger::formatted_builder()
        .filter_module("minimal", log::LevelFilter::Info)
        .filter_module("bevy", log::LevelFilter::Info)
        .filter_module("bevy_headless", log::LevelFilter::Info)
        .init();

    let (w, h) = (1920, 1080);

    Engine::new()
        // .insert_resource(CurrImage::default())
        .insert_resource(bevy_headless::SceneInfo::new(w, h))
        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugins((
            ImageExportPlugin,
            ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, save_img)
        .run();
}
