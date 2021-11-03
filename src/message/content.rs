use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
pub enum Content {
    Text(String),
    Image {
        url: String,
        width: f64,
        height: f64,
    },
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Text(text) => text,
            Self::Image { url, .. } => url,
        };
        f.write_str(str)
    }
}
