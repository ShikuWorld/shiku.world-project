#[macro_use]
extern crate diesel;
extern crate thiserror;

use dotenv::dotenv;
use env_logger::Builder;
use log::{debug, LevelFilter};
use std::io::Write;
use std::time::Duration;

use crate::conductor_module::def::ConductorModule;
use crate::core::blueprint::def::{BlueprintError, BlueprintService};
use crate::core::blueprint::resource_cache::init_resource_cache;
use crate::core::module::SystemModule;
use crate::core::{blueprint, TARGET_FPS};
use crate::resource_module::def::ResourceModule;
use crate::websocket_module::WebsocketModule;

mod conductor_module;
mod core;
mod login;
mod persistence_module;
mod resource_module;
mod webserver_module;
mod websocket_module;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}|{}] - {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Error)
        .filter(Some("home"), LevelFilter::Debug)
        .filter(Some("hyper"), LevelFilter::Error)
        .init();

    init_resource_cache().expect("Resource cache should initialize without problems.");

    let mut websocket_module = WebsocketModule::new();
    websocket_module.start();

    let blueprint_service =
        BlueprintService::create().expect("Could not create blueprint service!");

    let conductor_blueprint = match BlueprintService::load_conductor_blueprint() {
        Ok(b) => b,
        Err(BlueprintError::FileDoesNotExist(_)) => blueprint::def::Conductor::default(),
        Err(err) => panic!("{:?}", err),
    };

    BlueprintService::save_conductor_blueprint(&conductor_blueprint)
        .expect("Initial saving of conductor blueprint failed!");

    let mut conductor_module =
        ConductorModule::new(websocket_module, blueprint_service, conductor_blueprint).await;

    let mut interval = spin_sleep_util::interval(Duration::from_secs(1) / TARGET_FPS as u32);
    debug!("Starting main loop.");
    loop {
        conductor_module.conduct().await;

        interval.tick();
    }
}
