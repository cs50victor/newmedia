use bevy_ws_server::{ReceiveError, WsConnection, WsListener};
use log::info;
use serde_json::json;

use std::fmt::Debug;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_frame_capture::scene::CurrImageBase64;

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
    info!("starting HTTP server on port {port}");

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
    curr_base64_img: Res<CurrImageBase64>,
    connections: Query<(Entity, &WsConnection)>,
) {
    for (entity, conn) in connections.iter() {
        loop {
            match conn.receive() {
                Ok(message) => {
                    info!("message | {message:?}");
                    let resp = tungstenite::protocol::Message::Text(
                        json!({
                            "image": curr_base64_img.0
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
