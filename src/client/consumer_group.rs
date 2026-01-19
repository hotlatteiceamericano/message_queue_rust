use std::{collections::HashMap, io};

use crate::{message::Message, topic::topic::Topic};

pub struct ConsumerGroup {
    name: String,
    read_offset: HashMap<Topic, u64>,
}

impl ConsumerGroup {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
        let message = topic.read(offset)?;

        self.read_offset
            .entry(topic)
            .and_modify(|v| *v += message.total_length() as u64);

        Ok(message)
    }
}

#[cfg(test)]
mod test {
    use rstest::{fixture, rstest};

    use crate::client::consumer_group::ConsumerGroup;
    use crate::test_utils::TestTopic;

    #[fixture]
    fn test_topic() -> TestTopic {
        use crate::test_utils::TestTopic;
        TestTopic::new()
    }

    #[rstest]
    pub fn test_write_poll(test_topic: TestTopic) {
        let mut consumer_group = ConsumerGroup::new("test_consumer_group");
        // needed to call save as ConsumerGroup::write finds the topic from local file
        // todo: find the way to not call the save()
        test_topic.topic.save().unwrap();

        consumer_group
            .write(
                test_topic.topic.name().to_string(),
                String::from("hello world!"),
            )
            .unwrap();

        let message = consumer_group
            .poll(test_topic.topic.name().to_string())
            .unwrap();

        assert_eq!(message.content, "hello world!");
    }

    #[rstest]
    pub fn test_read_offset(test_topic: TestTopic) {
        let mut consumer_group = ConsumerGroup::new("test_consumer_group");
        test_topic.topic.save().unwrap();

        consumer_group
            .write(
                test_topic.topic.name().to_string(),
                String::from("hello world 1"),
            )
            .unwrap();
        consumer_group
            .write(
                test_topic.topic.name().to_string(),
                String::from("hello world 2"),
            )
            .unwrap();

        let first_message = consumer_group
            .poll(test_topic.topic.name().to_string())
            .unwrap();

        assert_eq!(first_message.content, "hello world 1");

        let second_message = consumer_group
            .poll(test_topic.topic.name().to_string())
            .unwrap();

        assert_eq!(second_message.content, "hello world 2");
    }
}
