use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Handshake {
    pub token: String,
}

impl Handshake {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HandshakeReply {
    pub success: bool,
    pub message: Option<String>,
}

impl HandshakeReply {
    pub fn new(success: bool, message: Option<String>) -> Self {
        Self { success, message }
    }

    pub fn success(message: Option<String>) -> Self {
        Self::new(true, message)
    }

    pub fn failed(message: Option<String>) -> Self {
        Self::new(false, message)
    }

    pub fn error(err: impl std::error::Error) -> Self {
        Self::failed(Some(err.to_string()))
    }
}
