use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub content: String,
}

impl Message {
    const MESSAGE_SIZE: u64 = 128;

    pub fn new(content: String) -> Self {
        Self { content }
    }
}
