#[macro_use]
extern crate diesel;
extern crate thiserror;

use std::io::Write;

use dotenv::dotenv;
use env_logger::Builder;
use log::LevelFilter;
use spin_sleep::LoopHelper;

use crate::conductor_module::def::ConductorModule;
use crate::core::{blueprint, TARGET_FPS};
use crate::core::blueprint::def::{BlueprintError, BlueprintService};
use crate::core::blueprint::resource_cache::init_resource_cache;
use crate::core::module::SystemModule;
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
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .filter(None, LevelFilter::Debug)
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

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(TARGET_FPS);

    loop {
        loop_helper.loop_start();

        conductor_module.conduct().await;

        loop_helper.loop_sleep();
    }
}
