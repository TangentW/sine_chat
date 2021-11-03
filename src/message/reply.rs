use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReply {
    pub success: bool,
    pub message: Option<String>,
    pub extra: Option<Map<String, Value>>,
}

impl MessageReply {
    pub fn new(success: bool, message: Option<String>, extra: Option<Map<String, Value>>) -> Self {
        Self {
            success,
            message,
            extra,
        }
    }

    pub fn success(message: Option<String>) -> Self {
        Self::new(true, message, None)
    }

    pub fn failed(message: Option<String>) -> Self {
        Self::new(false, message, None)
    }

    pub fn error(err: impl std::error::Error) -> Self {
        Self::failed(Some(err.to_string()))
    }

    pub fn put_extra(&mut self, key: impl ToString, value: Value) -> &mut Self {
        self.get_or_new_extra().insert(key.to_string(), value);
        self
    }

    fn get_or_new_extra(&mut self) -> &mut Map<String, Value> {
        if self.extra.is_none() {
            self.extra = Some(Map::new());
        }
        self.extra.as_mut().unwrap()
    }
}
