use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
use tokio::sync::{mpsc, RwLock};
use warp::fs;
use warp::ws::Message;
use warp::Filter;

use ws::{with_clients, ws_handler, Clients};

mod ws;

#[tokio::main]
async fn main() {
    let static_files = warp::path("static").and(fs::dir("static"));
    let clients: Clients = Arc::from(RwLock::from(HashMap::new()));

    let ws_handler = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(ws_handler);

    let routes = static_files.or(ws_handler);
    let (tx, mut rx) = mpsc::channel(50);

    let mut watcher = recommended_watcher(move |res| {
        if let Err(err) = tx.blocking_send(res) {
            println!("Could not send message {err}");
        }
    })
    .unwrap();

    watcher
        .watch(Path::new("./static"), RecursiveMode::Recursive)
        .unwrap();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Ok(event) = message {
                let current_clients = clients.write().await;
                for client in current_clients.values() {
                    if let EventKind::Modify(d) = event.kind {
                        if let Some(full_path) =
                            event.paths.get(0).unwrap_or(&PathBuf::new()).to_str()
                        {
                            let search_str = "./static/";
                            if let Some(index) = full_path.find(search_str) {
                                let start_index = index + search_str.len();
                                let relative_path = &full_path[start_index..];
                                if let Err(err) = client.sender.send(Ok(Message::text(format!(
                                    "{{\"path\": {:?}, \"kind\": {:?}}}",
                                    relative_path, d
                                )))) {
                                    eprintln!("Could not send message {:?}", err);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    warp::serve(routes).run(([0, 0, 0, 0], 8083)).await;
}
