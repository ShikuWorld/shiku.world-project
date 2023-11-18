use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use futures::FutureExt;
use futures::StreamExt;
use notify::event::ModifyKind;
use serde::Serialize;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::Message;
use warp::{
    ws::{WebSocket, Ws},
    Reply,
};
use warp::{Filter, Rejection};

#[derive(Debug)]
pub struct Client {
    pub sender: mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
}

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

type Result<T> = std::result::Result<T, Rejection>;

pub async fn ws_handler(ws: Ws, clients: Clients) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| handle_websocket(socket, clients)))
}

pub fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

#[derive(Debug, Serialize)]
pub struct PicUpdateEvent {
    pub path: String,
}

async fn handle_websocket(websocket: WebSocket, clients: Clients) {
    let (client_ws_sender, mut client_ws_rcv) = websocket.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));
    let id = Uuid::new_v4().as_simple().to_string();
    clients.write().await.insert(
        id.clone(),
        Client {
            sender: client_sender,
        },
    );

    println!("connected");

    while let Some(result) = client_ws_rcv.next().await {
        match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}
