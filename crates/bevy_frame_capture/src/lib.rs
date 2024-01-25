/// Derived from: https://github.com/bevyengine/bevy/pull/5550
mod image_copy;
mod scene;

use bevy::app::{App, Plugin, PostUpdate};
use image_copy::ImageCopyPlugin;
use scene::update;
pub use scene::{SceneInfo,setup_render_target, white_img_placeholder, CurrImageBase64};

pub struct FrameCapturePlugin;

impl Plugin for FrameCapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ImageCopyPlugin);
        app.add_systems(PostUpdate, update);
        app.init_resource::<scene::SceneInfo>();
        app.add_event::<scene::SceneInfo>();
    }
}