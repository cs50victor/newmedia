use log::info;

use actix_web::web;

use std::fmt::Debug;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerMsg<T> {
    data: Option<T>,
    error: Option<String>,
}

impl<T: ToString> ServerMsg<T> {
    pub fn data(data: T) -> Self {
        Self { data: Some(data), error: None }
    }

    pub fn error(error: T) -> Self {
        let err_msg = error.to_string();
        log::warn!("Server error. {err_msg:?}");
        Self { data: None, error: Some(err_msg) }
    }
}

#[derive(Clone, Resource)]
pub struct ShutdownBevyRemotely {
    tx: crossbeam_channel::Sender<bool>,
    rx: crossbeam_channel::Receiver<bool>,
}

impl FromWorld for ShutdownBevyRemotely {
    fn from_world(_world: &mut World) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<bool>();
        Self { tx, rx }
    }
}

pub fn shutdown_bevy_remotely(
    mut app_exit_writer: EventWriter<bevy::app::AppExit>,
    shutdown: ResMut<ShutdownBevyRemotely>,
) {
    if let Ok(true) = shutdown.rx.try_recv() {
        log::info!("received bevy shutdown signal");
        app_exit_writer.send(bevy::app::AppExit);
    }
}

pub struct RoomData {
    pub stream_frame_data: crate::StreamingFrameData,
}

mod health_check {
    pub async fn handler() -> impl actix_web::Responder {
        actix_web::HttpResponse::Ok().json(super::ServerMsg::data("OK"))
    }
}

pub type ServerStateMutex = parking_lot::Mutex<ServerResources>;

fn top_level_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").service(web::resource("").to(health_check::handler)));
}

pub struct ServerResources {
    pub app_state: std::sync::Arc<parking_lot::Mutex<crate::ParticipantRoomName>>,
}

pub async fn http_server(
    tx: crossbeam_channel::Sender<actix_web::dev::ServerHandle>,
    app_state: std::sync::Arc<parking_lot::Mutex<crate::ParticipantRoomName>>,
) -> std::io::Result<()> {
    // let _ =  setAppState;
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "6669".to_string())
        .parse::<u16>()
        .expect("PORT couldn't be set");

    info!("starting HTTP server on port {port}");

    let server_resources =
        actix_web::web::Data::new(parking_lot::Mutex::new(ServerResources { app_state }));

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::new("IP - %a | Time - %D ms"))
            .wrap(
                actix_web::middleware::DefaultHeaders::new()
                    .add(("Content-Type", "application/json")),
            )
            .app_data(server_resources.clone())
            .configure(top_level_routes)
    })
    .bind(("0.0.0.0", port))?
    .workers(1)
    .run();

    // server
    let _ = tx.send(server.handle());

    server.await
}

#[derive(Resource)]
pub struct ActixServer {
    server_handle: actix_web::dev::ServerHandle,
}

impl bevy::ecs::world::FromWorld for ActixServer {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<ShutdownBevyRemotely>();

        let app_state =
            std::sync::Arc::new(parking_lot::Mutex::new(crate::ParticipantRoomName::default()));

        world.insert_resource(crate::AppStateSync { state: app_state.clone(), dirty: false });

        let async_runtime = world.get_resource::<crate::AsyncRuntime>().unwrap();

        let (tx, rx) = crossbeam_channel::unbounded::<actix_web::dev::ServerHandle>();

        let shutdown_bev = world.get_resource::<ShutdownBevyRemotely>().unwrap();
        let shutdown_bev_tx = shutdown_bev.tx.clone();

        log::info!("spawning thread for server");

        let rt = async_runtime.rt.clone();

        std::thread::spawn(move || {
            let svr = http_server(tx, app_state);
            if let Err(e) = rt.block_on(svr) {
                log::info!("Server errored out | Reason {e:#?}");
            };
            log::warn!("Server exited  | Shutting down Bevy");
            shutdown_bev_tx.send(true).unwrap();
        });

        let server_handle = rx.recv().unwrap();

        Self { server_handle }
    }
}
