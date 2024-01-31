use bevy::{
    ecs::{
        query::With,
        system::{Query, Res, Resource},
        world::{FromWorld, World},
    },
    render::camera::Camera,
    transform::components::Transform,
};

/// Allows LLM / Model to Control the Bevy World remotely
#[derive(Resource)]
pub struct WorldControlChannel {
    pub tx: crossbeam_channel::Sender<String>,
    rx: crossbeam_channel::Receiver<String>,
}

impl FromWorld for WorldControlChannel {
    fn from_world(_: &mut World) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<String>();
        Self { tx, rx }
    }
}

pub fn update_world_from_input(
    input_receiver: Res<WorldControlChannel>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let speed = 1.0;
    if let Ok(input) = input_receiver.rx.try_recv() {
        log::info!("user input : {input}");
        match input.as_str() {
            "UP" => camera.iter_mut().for_each(|mut transform| transform.translation.y += speed),
            "DOWN" => camera.iter_mut().for_each(|mut transform| transform.translation.y -= speed),
            "LEFT" => camera.iter_mut().for_each(|mut transform| transform.translation.x -= speed),
            "RIGHT" => camera.iter_mut().for_each(|mut transform| transform.translation.x += speed),
            "ZOOM-IN" => {
                camera.iter_mut().for_each(|mut transform| transform.translation.z -= speed)
            },
            "ZOOM-OUT" => {
                camera.iter_mut().for_each(|mut transform| transform.translation.z += speed)
            },
            e => {
                log::info!("Input received | {e}");
            },
        }
    }
}
