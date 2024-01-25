use bevy::{
    app::ScheduleRunnerPlugin, core_pipeline::tonemapping::Tonemapping, prelude::*,
    render::renderer::RenderDevice,
};


pub struct AppConfig {
    width: u32,
    height: u32,
}

fn headless_app() {
    let mut app = App::new();

    let config = AppConfig {
        width: 1920,
        height: 1080,
    };

    // setup frame capture
    app.insert_resource(bevy_frame_capture::scene::SceneController::new(
        config.width,
        config.height,
    ));

    app.insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)));

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                close_when_requested: false,
            }),
    );

    app.add_plugins(bevy_frame_capture::image_copy::ImageCopyPlugin);

    // headless frame capture
    app.add_plugins(bevy_frame_capture::scene::CaptureFramePlugin);

    app.add_plugins(ScheduleRunnerPlugin::run_loop(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));

    app.init_resource::<bevy_frame_capture::scene::SceneController>();
    app.add_event::<bevy_frame_capture::scene::SceneController>();

    app.add_systems(Startup, setup);

    app.run();
}

pub fn main() {
    headless_app();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<bevy_frame_capture::scene::SceneController>,
    render_device: Res<RenderDevice>,
) {
    let render_target = bevy_frame_capture::scene::setup_render_target(
        &mut commands,
        &mut images,
        &render_device,
        &mut scene_controller,
        15,
        String::from("main_scene"),
    );

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        tonemapping: Tonemapping::None,
        camera: Camera {
            target: render_target,
            ..default()
        },
        ..default()
    });
}