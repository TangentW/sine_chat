use std::{sync::Arc, time::Duration};

use guard::guard;
use log::{error, info};
use tokio::{net::TcpStream, select, sync::mpsc, task::JoinHandle, time};

use crate::{
    frame,
    message::{ClientMessage, Handshake, HandshakeReply, Ping},
};

use super::{Client, Clients, Entry, Item, Reader, Receiver, Sender, Writer};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct ClientTask {
    clients: Clients,
    sender: Sender,
    client: Option<Arc<Client>>,
    sending_task: Option<JoinHandle<()>>,
}

impl ClientTask {
    pub async fn run(stream: TcpStream, entry: Entry, clients: Clients) {
        let (sender, receiver) = mpsc::channel(256);
        let mut task = ClientTask {
            clients,
            sender,
            client: None,
            sending_task: None,
        };
        let (mut reader, mut writer) = Self::split(stream);
        // Step 1: handshake
        if !task.handshake(&mut reader, &mut writer).await {
            return;
        }
        // Step 2: run loop
        task.run_sending(receiver, writer);
        task.run_receiving(reader, entry).await;
    }
}

impl ClientTask {
    fn split(stream: TcpStream) -> (Reader, Writer) {
        let (reader, writer) = stream.into_split();
        (Reader::new(reader), Writer::new(writer))
    }
}

// Handshake

impl ClientTask {
    async fn handshake(&mut self, reader: &mut Reader, writer: &mut Writer) -> bool {
        let handshake = select! {
            _ = time::sleep(HANDSHAKE_TIMEOUT) => None,
            handshake = reader.read::<Handshake>() => handshake,
        };
        guard!(let Some(handshake) = handshake else { return false });

        let (success, reply) = self.process_handshake(handshake);
        if let Err(err) = writer.write(reply).await {
            error!("Writer error: {}", err);
        }
        if success {
            info!("Client connected: {}", self.client.as_ref().unwrap().uid);
        }
        success
    }

    fn process_handshake(&mut self, handshake: frame::Result<Handshake>) -> (bool, HandshakeReply) {
        let mut clients = self.clients.lock().unwrap();

        let (token, reply) = match handshake {
            Ok(handshake) => {
                if clients.contains_key(&handshake.token) {
                    (
                        None,
                        HandshakeReply::failed(Some("User existed".to_string())),
                    )
                } else {
                    (Some(handshake.token), HandshakeReply::success(None))
                }
            }
            Err(err) => (None, HandshakeReply::error(err)),
        };

        self.client = token.map(|uid| {
            let client = Arc::new(Client::new(uid.clone(), self.sender.clone()));
            clients.insert(uid, client.clone());
            client
        });

        let success = self.client.is_some();
        (success, reply)
    }
}

// Sending & Receiving

impl ClientTask {
    fn run_sending(&mut self, mut receiver: Receiver, mut writer: Writer) {
        let task = tokio::spawn(async move {
            loop {
                guard!(let Some(msg) = receiver.recv().await else { break });
                if let Err(err) = writer.write(msg).await {
                    error!("Writer error: {}", err);
                }
            }
        });
        self.sending_task = Some(task);
    }

    async fn run_receiving(&self, mut reader: Reader, entry: Entry) {
        guard!(let Some(client) = self.client.clone() else { return });
        while let Some(msg) = reader.read_either::<ClientMessage, Ping>().await {
            let item = Item::new(client.clone(), msg);
            entry.send(item).await.unwrap();
        }
    }
}

// Drop

impl Drop for ClientTask {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            info!("Client disconnected: {}", client.uid);
            self.clients.lock().unwrap().remove(&client.uid);
        }
        if let Some(sending_task) = self.sending_task.take() {
            sending_task.abort();
        }
    }
}
