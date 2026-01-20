use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub content: String,
}

impl Message {
    pub const MESSAGE_LENGTH: u64 = 4;

    pub fn new(content: &str) -> Self {
        Self {
            content: String::from(content),
        }
    }

    pub fn content_length(&self) -> u64 {
        bincode::serialized_size(self).unwrap()
    }

    pub fn total_length(&self) -> u64 {
        self.content_length() + Self::MESSAGE_LENGTH
    }
}
