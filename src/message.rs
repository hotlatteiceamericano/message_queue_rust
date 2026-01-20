use std::io;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub content: String,
}

impl Message {
    pub const MESSAGE_LENGTH: u32 = 4;

    pub fn new(content: &str) -> Self {
        Self {
            content: String::from(content),
        }
    }

    pub fn content_length(&self) -> u32 {
        bincode::serialize(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .unwrap()
            .len() as u32
    }

    pub fn total_length(&self) -> u32 {
        self.content_length() + Self::MESSAGE_LENGTH
    }
}
