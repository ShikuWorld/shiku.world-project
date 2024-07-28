#[macro_use]
extern crate diesel;
extern crate thiserror;

use crate::conductor_module::def::ConductorModule;
use crate::core::blueprint::def::{BlueprintError, BlueprintService};
use crate::core::blueprint::resource_cache::init_resource_cache;
use crate::core::module::SystemModule;
use crate::core::{blueprint, TARGET_FPS};
use crate::log_collector::LogCollector;
use crate::resource_module::def::ResourceModule;
use crate::websocket_module::WebsocketModule;
use dotenv::dotenv;
use fern::Output;
use log::{debug, LevelFilter, Record};
use std::cell::RefCell;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

mod conductor_module;
mod core;
mod log_collector;
mod login;
mod persistence_module;
mod resource_module;
mod webserver_module;
mod websocket_module;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let log_collector = Arc::new(LogCollector::new());
    let collector_clone = Arc::clone(&log_collector);
    let collector_output = Output::call(move |record: &Record| {
        collector_clone.log(
            humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
            record.level().to_string(),
            record.target().to_string(),
            record.args().to_string(),
        );
    });
    let standard_output = Output::call(|record: &Record| {
        println!(
            "[{} {} {}] {}",
            humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
            record.level(),
            record.target(),
            record.args()
        );
    });

    fern::Dispatch::new()
        .format(|out, message, _| {
            out.finish(format_args!("{}", message));
        })
        .level(LevelFilter::Debug)
        .level_for("home", LevelFilter::Debug)
        .level_for("hyper", LevelFilter::Error)
        .chain(standard_output)
        .chain(collector_output)
        .apply()
        .expect("Could not apply logger!");

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

    let mut conductor_module = ConductorModule::new(
        websocket_module,
        blueprint_service,
        conductor_blueprint,
        log_collector,
    )
    .await;

    let mut interval = spin_sleep_util::interval(Duration::from_secs(1) / TARGET_FPS as u32);
    debug!("Starting main loop.");
    loop {
        conductor_module.conduct().await;

        interval.tick();
    }
}
