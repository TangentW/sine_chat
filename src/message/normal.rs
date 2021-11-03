use serde::{Deserialize, Serialize};

use super::Content;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientMessage {
    pub content: Content,
    pub receiver: String,
}

impl ClientMessage {
    pub fn new(content: Content, receiver: String) -> Self {
        Self { content, receiver }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerMessage {
    #[serde(flatten)]
    pub content: Content,
    pub sender: String,
    pub receiver: String,
}

impl ServerMessage {
    pub fn new(content: Content, sender: String, receiver: String) -> Self {
        Self {
            content,
            sender,
            receiver,
        }
    }
}
