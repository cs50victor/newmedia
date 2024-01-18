use bevy::ecs::system::Resource;

pub struct ReceivedVideoFrame {
    pub image_buffer: Vec<u8>,
    pub timestamp: i64, // When the frame was captured in microseconds
}

#[derive(Resource)]
pub struct VideoChannel {
    pub tx: crossbeam_channel::Sender<Vec<i16>>,
    rx: crossbeam_channel::Receiver<Vec<i16>>,
}

impl Default for VideoChannel {
    fn default() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<Vec<i16>>();
        Self { tx, rx }
    }
}
impl VideoChannel {
    pub fn new() -> Self {
        Self::default()
    }
}
