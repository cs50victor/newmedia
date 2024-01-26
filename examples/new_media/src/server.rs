use bevy::ecs::{
    entity::Entity,
    system::{Commands, Query, Res},
};
use bevy_headless::CurrImageContainer;
use bevy_ws_server::{ReceiveError, WsConnection, WsListener};
use log::info;
use serde_json::json;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServerMsg<T> {
    data: Option<T>,
    error: Option<String>,
}

impl<T: ToString> HttpServerMsg<T> {
    pub fn data(data: T) -> Self {
        Self { data: Some(data), error: None }
    }

    pub fn error(error: T) -> Self {
        let err_msg = error.to_string();
        log::warn!("Server error. {err_msg:?}");
        Self { data: None, error: Some(err_msg) }
    }
}

pub fn start_ws(listener: Res<WsListener>) {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT couldn't be set");

    match listener.listen(([127, 0, 0, 1], port), None) {
        Ok(host) => {
            log::info!("Websocket Server listening on {host:?}");
        },
        Err(e) => {
            log::error!("error starting WS listener: {e}");
        },
    };
}

pub fn receive_message(
    mut commands: Commands,
    curr_image: Res<CurrImageContainer>,
    connections: Query<(Entity, &WsConnection)>,
) {
    for (entity, conn) in connections.iter() {
        loop {
            match conn.receive() {
                Ok(message) => {
                    info!("message | {message:?}");
                    let resp = tungstenite::protocol::Message::Text(
                        json!({
                            "image": curr_image.0.lock().to_web_base64().unwrap()
                        })
                        .to_string(),
                    );
                    conn.send(resp);
                },
                Err(ReceiveError::Empty) => break,
                Err(ReceiveError::Closed) => {
                    commands.entity(entity).despawn();
                    break;
                },
            }
        }
    }
}
