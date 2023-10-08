use crate::core::module::{ModuleName, ModuleState, SystemModule};
use flume::{unbounded, Receiver, RecvError, Sender};
use futures::stream::{SplitSink, SplitStream};
use futures::{future, SinkExt, StreamExt, TryStreamExt};
use log::debug;
use log::error;
use log::trace;

use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdBucket;
use std::collections::HashMap;
use std::vec::Drain;
use ts_rs::TS;

use crate::core::guest::SessionId;
use crate::core::Snowflake;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Error as WsError, Message};

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export)]
pub struct Ticket {
    pub session_id: Option<SessionId>,
    pub admin_login: Option<bool>,
}

#[derive(Debug)]
pub struct WSConnection {
    pub connection_id: Snowflake,
    pub ticket: Option<Ticket>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

pub struct WebsocketModule {
    module_state: ModuleState,
    pub connections: HashMap<Snowflake, WSConnection>,
    new_connection_receiver: Option<Receiver<WSConnection>>,
    new_connections: Vec<Snowflake>,
    lost_connections: Vec<Snowflake>,
}

impl WebsocketModule {
    pub fn new() -> WebsocketModule {
        WebsocketModule {
            module_state: ModuleState::Stopped,
            connections: HashMap::new(),
            new_connection_receiver: None,
            new_connections: Vec::new(),
            lost_connections: Vec::new(),
        }
    }

    pub fn handle_new_ws_connections(&mut self) -> Vec<(Snowflake, Ticket)> {
        if let Some(receiver) = &self.new_connection_receiver {
            for guest_connection in receiver.drain() {
                self.new_connections.push(guest_connection.connection_id);
                self.connections
                    .insert(guest_connection.connection_id, guest_connection);
            }
        }

        Self::drain_filter_new_connections_on_ticket(
            &mut self.new_connections,
            &mut self.connections,
        )
    }

    pub fn drain_filter_new_connections_on_ticket(
        new_connections: &mut Vec<Snowflake>,
        connections: &mut HashMap<Snowflake, WSConnection>,
    ) -> Vec<(Snowflake, Ticket)> {
        let (drained_cons, rest_cons): (Vec<Snowflake>, Vec<Snowflake>) =
            new_connections.iter().partition(|connection_id| {
                if let Some(connection) = connections.get_mut(connection_id) {
                    for message in connection.receiver.drain() {
                        match serde_json::from_str::<Ticket>(message.to_string().as_str()) {
                            Ok(ticket) => {
                                debug!(
                                    "Got ticket from {}!",
                                    if let Some(true) = ticket.admin_login {
                                        "admin"
                                    } else {
                                        "guest"
                                    }
                                );
                                connection.ticket = Some(ticket);
                                return true;
                            }
                            Err(err) => {
                                error!("Could not parse ticket. {:?}", err);
                            }
                        }
                    }
                }
                false
            });

        *new_connections = rest_cons;

        drained_cons
            .into_iter()
            .filter_map(|connection_id| {
                if let Some(c) = connections.get(&connection_id) {
                    if let Some(ticket) = &c.ticket {
                        return Some((connection_id, ticket.clone()));
                    }
                }
                None
            })
            .collect()
    }

    pub fn drain_lost_connections(&mut self) -> Drain<Snowflake> {
        self.lost_connections.drain(..)
    }

    pub fn drop_lost_ws_connections(&mut self) {
        for (connection_id, connection) in &self.connections {
            if connection.receiver.is_disconnected() {
                self.lost_connections.push(*connection_id);
            }
        }

        for connection_id in &self.lost_connections {
            self.connections.remove(connection_id);
        }
    }

    pub fn send_event(&mut self, ws_connection_id: &Snowflake, event: String) {
        trace!("Sending event to {}", ws_connection_id);
        if let Some(connection) = self.connections.get_mut(ws_connection_id) {
            match connection.sender.try_send(Message::from(event)) {
                Ok(_) => {}
                Err(err) => {
                    error!("{:?}", err);
                }
            }
        } else {
            error!(
                "Could not send to connection_id {} because connection doesn't exist!",
                ws_connection_id
            );
        }
    }

    pub fn drain_events(&mut self, guest_id: &Snowflake) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();
        if let Some(connection) = self.connections.get_mut(guest_id) {
            for message in connection.receiver.drain() {
                messages.push(message);
            }
        }
        messages
    }
}

impl SystemModule for WebsocketModule {
    fn module_name(&self) -> ModuleName {
        String::from("WebsocketModule")
    }

    fn status(&self) -> &ModuleState {
        &self.module_state
    }

    fn start(&mut self) {
        debug!("Starting websocket module");
        self.module_state = ModuleState::Starting;
        let (connection_sender, connection_receiver) = unbounded();
        spawn_websocket_server(connection_sender);
        self.new_connection_receiver = Some(connection_receiver);
    }

    fn shutdown(&mut self) {
        debug!("Shutdown not implemented. :P");
    }
}

fn spawn_websocket_server(connection_sender: Sender<WSConnection>) {
    debug!("spawn_websocket_server");
    tokio::spawn(async move {
        debug!("TCP Thread spawned.");

        let server_result = TcpListener::bind("0.0.0.0:9001").await;

        debug!("Websocket Server running at 9001.");

        match server_result {
            Ok(server) => {
                let mut connection_id_generator = SnowflakeIdBucket::new(1, 1);

                while let Ok((stream, _)) = server.accept().await {
                    debug!("New connection!");
                    let guest_connection_result =
                        setup_ws_connection(connection_id_generator.get_id(), stream).await;
                    match guest_connection_result {
                        Ok(guest_connection) => {
                            match connection_sender.send_async(guest_connection).await {
                                Ok(_) => {
                                    trace!("Connection send!");
                                }
                                Err(err) => {
                                    error!("{:?}", err);
                                }
                            }
                        }
                        Err(err) => {
                            error!("{:?}", err);
                        }
                    }
                }
                debug!("no more connections...?");
            }
            Err(err) => {
                error!("Error while accepting connection {:?}", err);
            }
        }
    });
}

async fn setup_ws_connection(
    connection_id: Snowflake,
    stream: TcpStream,
) -> Result<WSConnection, WsError> {
    debug!("Setting up ws connection");
    let websocket_stream = tokio_tungstenite::accept_async(stream).await?;
    let (ws_out_sender, ws_out_receiver) = unbounded();
    let (ws_in_sender, ws_in_receiver) = unbounded();

    let (outgoing, incoming) = websocket_stream.split();
    setup_sending_messages_to_websocket(outgoing, ws_out_receiver);
    setup_reading_messages_from_websocket(incoming, ws_in_sender);

    Ok(WSConnection {
        connection_id,
        ticket: None,
        sender: ws_out_sender,
        receiver: ws_in_receiver,
    })
}

fn setup_reading_messages_from_websocket(
    incoming: SplitStream<WebSocketStream<TcpStream>>,
    ws_in_sender: Sender<Message>,
) {
    tokio::spawn(async move {
        let for_each_result = incoming
            .try_for_each(|msg| {
                if msg.is_binary() || msg.is_text() {
                    match ws_in_sender.send(msg) {
                        Ok(()) => (),
                        Err(err) => {
                            error!("Error sending message to inbox {:?}", err);
                        }
                    }
                }
                future::ok(())
            })
            .await;
        match for_each_result {
            Ok(()) => {
                debug!("WS Reading thread unwound properly.");
            }
            Err(err) => error!("Error while unwinding read stream {:?}", err),
        }
    });
}

fn setup_sending_messages_to_websocket(
    mut outgoing: SplitSink<WebSocketStream<TcpStream>, Message>,
    ws_out_receiver: Receiver<Message>,
) {
    tokio::spawn(async move {
        'thread_loop: loop {
            match ws_out_receiver.recv_async().await {
                Ok(message) => {
                    match outgoing.send(message).await {
                        Ok(_) => {
                            trace!("Sent message?");
                        }
                        // TODO: Properly handle errors
                        Err(err) => {
                            error!("Something went wrong while sending messages {:?}", err);
                            break 'thread_loop;
                        }
                    }
                }
                Err(RecvError::Disconnected) => {
                    debug!("WS Sending thread unwound properly.");
                    break 'thread_loop;
                }
            }
        }
    });
}
