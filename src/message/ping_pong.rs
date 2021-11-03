use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Ping;

impl Ping {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Ping {
    fn default() -> Self {
        Self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Pong;

impl Pong {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Pong {
    fn default() -> Self {
        Self
    }
}
