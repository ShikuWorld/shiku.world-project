use std::convert::Infallible;
use std::env;
use std::sync::Arc;

use tokio::sync::Mutex;
use warp::Filter;

use crate::webserver_module::def::{DoorStatuses, WebServerModule};
use crate::SystemModule;

impl SystemModule for WebServerModule {
    fn start(&mut self) {
        todo!()
    }
}

impl WebServerModule {
    pub fn new() -> WebServerModule {
        let mut cors = warp::cors().allow_methods(vec!["GET", "POST", "DELETE"]);

        for cors_origin in env::var("RESOURCE_SERVER_CORS").unwrap().split('|') {
            cors = cors.allow_origin(cors_origin);
        }

        let door_statuses = Arc::new(Mutex::new(DoorStatuses {
            main_door_status: false,
            back_door_status: false,
        }));
        let door_statuses_raw = DoorStatuses {
            main_door_status: false,
            back_door_status: false,
        };

        let main_door_status_get = {
            let door_statuses = door_statuses.clone();
            warp::path("main-door-status")
                .and(warp::get())
                .and_then(move || WebServerModule::return_main_door_status(door_statuses.clone()))
        };

        let back_door_status_get = {
            let door_statuses = door_statuses.clone();
            warp::path("back-door-status")
                .and(warp::get())
                .and_then(move || WebServerModule::return_main_door_status(door_statuses.clone()))
        };

        tokio::spawn(async move {
            warp::serve(main_door_status_get.or(back_door_status_get))
                .run(([0, 0, 0, 0], 3030))
                .await;
        });

        WebServerModule {
            door_statuses,
            door_statuses_raw,
        }
    }

    pub async fn set_main_door_status(&mut self, status: bool) {
        self.door_statuses_raw.main_door_status = status;
        let mut statuses = self.door_statuses.lock().await;
        statuses.main_door_status = status;
    }

    pub async fn set_back_door_status(&mut self, status: bool) {
        self.door_statuses_raw.back_door_status = status;
        let mut statuses = self.door_statuses.lock().await;
        statuses.back_door_status = status;
    }

    pub async fn return_main_door_status(
        door_statuses: Arc<Mutex<DoorStatuses>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let lock = door_statuses.lock().await;
        Ok(warp::reply::html(format!("{}", lock.main_door_status)))
    }
}
