use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::info;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::either::Either;

use crate::{
    frame::{self, SendablePayload},
    message::{ClientMessage, MessageReply, Ping, Pong, ServerMessage},
};

mod client;
pub use self::client::Client;

mod client_task;
pub use self::client_task::ClientTask;

#[derive(Debug)]
pub struct Item {
    client: Arc<Client>,
    message: frame::Result<Either<ClientMessage, Ping>>,
}

impl Item {
    pub fn new(client: Arc<Client>, message: frame::Result<Either<ClientMessage, Ping>>) -> Self {
        Self { client, message }
    }
}

pub type Entry = mpsc::Sender<Item>;
pub type Clients = Arc<Mutex<HashMap<String, Arc<Client>>>>;
pub type Reader = frame::Reader<tokio::net::tcp::OwnedReadHalf>;
pub type Writer = frame::Writer<tokio::net::tcp::OwnedWriteHalf>;

type Sender = mpsc::Sender<Box<dyn SendablePayload>>;
type Receiver = mpsc::Receiver<Box<dyn SendablePayload>>;

pub struct Handler {
    entry: Entry,
    clients: Clients,
}

impl Handler {
    pub fn run() -> Handler {
        let (entry, receiver) = mpsc::channel(256);
        let handler = Self {
            entry,
            clients: Default::default(),
        };
        tokio::spawn(run(handler.clients.clone(), receiver));
        handler
    }

    pub fn connect(&self, stream: TcpStream) {
        let entry = self.entry.clone();
        let clients = self.clients.clone();
        tokio::spawn(ClientTask::run(stream, entry, clients));
    }
}

async fn run(clients: Clients, mut receiver: mpsc::Receiver<Item>) {
    while let Some(item) = receiver.recv().await {
        handle_item(item, clients.clone()).await;
    }
}

async fn handle_item(item: Item, clients: Clients) {
    match item.message {
        Ok(msg) => match msg {
            Either::Left(msg) => handle_message(msg, item.client, clients).await,
            Either::Right(_) => handle_ping(item.client).await,
        },
        Err(err) => handle_error(err, item.client).await,
    }
}

async fn handle_ping(sender: Arc<Client>) {
    sender.send(Pong).await
}

async fn handle_message(message: ClientMessage, sender: Arc<Client>, clients: Clients) {
    info!("Msg: {:?}", message);
    let receiver = clients.lock().unwrap().get(&message.receiver).cloned();
    if let Some(receiver) = receiver {
        // 1. Send reply to sender.
        let reply = MessageReply::success(None);
        sender.send(reply).await;
        // 2. Send message to sender & receiver.
        let message = ServerMessage::new(message.content, sender.uid.clone(), receiver.uid.clone());
        sender.send(message.clone()).await;
        receiver.send(message).await;
    } else {
        let reply = MessageReply::failed(Some("Receiver not found".to_string()));
        sender.send(reply).await
    }
}

async fn handle_error(err: frame::Error, sender: Arc<Client>) {
    let reply = MessageReply::failed(Some(err.to_string()));
    sender.send(reply).await
}
