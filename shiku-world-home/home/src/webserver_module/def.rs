use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Deserialize;

pub struct WebServerModule {
    pub door_statuses: Arc<Mutex<DoorStatuses>>,
}

pub struct DoorStatuses {
    pub main_door_status: bool,
    pub back_door_status: bool,
}

#[derive(Debug, Deserialize)]
pub struct DoorStatusUpdate {
    pub password: String,
    pub status: bool,
}
