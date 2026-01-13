use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub content: String,
}

impl Message {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
