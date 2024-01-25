use bevy::{
    app::ScheduleRunnerPlugin, core_pipeline::tonemapping::Tonemapping, prelude::*,
    render::renderer::RenderDevice,
};

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut scene_controller: ResMut<bevy_frame_capture::SceneInfo>,
    render_device: Res<RenderDevice>,
) {
    let render_target = bevy_frame_capture::setup_render_target(
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
        camera: Camera { target: render_target, ..default() },
        ..default()
    });
}

fn headless_app() {
    App::new()
        .insert_resource(bevy_frame_capture::SceneInfo::new(1920, 1080))
        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            close_when_requested: false,
        }))
        .add_plugins(bevy_frame_capture::FrameCapturePlugin)
        .add_plugins(ScheduleRunnerPlugin::run_loop(std::time::Duration::from_secs_f64(1.0 / 60.0)))
        .add_systems(Startup, setup)
        .run()
}

pub fn main() {
    headless_app();
}
