use std::{collections::HashMap, io};

use crate::{message::Message, topic::topic::Topic};

pub struct ConsumerGroup {
    name: String,
    read_offset: HashMap<Topic, u64>,
}

impl ConsumerGroup {
    pub fn new(name: String) -> Self {
        Self {
            name,
            read_offset: HashMap::new(),
        }
    }

    pub fn read_offset(&self) -> &HashMap<Topic, u64> {
        &self.read_offset
    }

    pub fn write(&mut self, topic_name: String, message: String) -> io::Result<()> {
        let mut topic = Topic::load(topic_name).expect("did not found topic");
        topic.write(&Message::new(message))?;

        if self.read_offset.get(&topic).is_none() {
            self.read_offset.insert(topic, 0);
        }

        Ok(())
    }

    pub fn poll(&mut self, topic_name: String) -> io::Result<Message> {
        let mut topic = Topic::load(topic_name)?;
        let offset = self.read_offset().get(&topic).unwrap();
        topic.read(offset.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::{client::consumer_group::ConsumerGroup, topic::topic::Topic};

    #[test]
    pub fn test_write_poll() {
        let mut consumer_group = ConsumerGroup::new(String::from("test_consumer_group"));
        let topic = Topic::new(String::from("consumer_group_testing_topic"));
        topic.save().unwrap();
        consumer_group
            .write(topic.name().to_string(), String::from("hello world!"))
            .unwrap();

        let message = consumer_group.poll(topic.name().to_string()).unwrap();

        assert_eq!(message.content, "hello world!");
    }
}
