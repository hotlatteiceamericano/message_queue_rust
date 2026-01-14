use std::{collections::HashMap, io};

use crate::topic::topic::Topic;

pub struct ConsumerGroup {
    name: String,
    topic_read_offset: HashMap<Topic, u64>,
}

impl ConsumerGroup {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic_read_offset: HashMap::new(),
        }
    }

    pub fn write(&mut self, topic_name: String) -> io::Result<()> {
        // find the topic with the name
        // call topic.write
        let topic = Topic::from(topic_name);
    }
}
