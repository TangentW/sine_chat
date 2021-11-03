use log::error;

use crate::frame::SendablePayload;

use super::Sender;

#[derive(Debug)]
pub struct Client {
    pub uid: String,
    sender: Sender,
}

impl Client {
    pub(crate) fn new(uid: String, sender: Sender) -> Self {
        Self { uid, sender }
    }

    pub async fn send<T>(&self, payload: T)
    where
        T: SendablePayload + 'static,
    {
        if let Err(err) = self.sender.send(Box::new(payload)).await {
            error!("Client sending error: {}", err);
        }
    }
}
