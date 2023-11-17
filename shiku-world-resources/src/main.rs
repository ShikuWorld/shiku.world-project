use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use notify::{recommended_watcher, RecursiveMode, Watcher};
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
                println!("{:?}", event);
                let current_clients = clients.write().await;
                for client in current_clients.values() {
                    if let Err(err) = client
                        .sender
                        .send(Ok(Message::text(format!("{:?}", event))))
                    {
                        eprintln!("Could not send message {:?}", err);
                    }
                }
            }
        }
    });

    warp::serve(routes).run(([127, 0, 0, 1], 8083)).await;
}
