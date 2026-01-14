use std::{collections::BTreeMap, io, path::PathBuf};

use crate::{message::Message, storage::segment::Segment};

pub struct Topic {
    segments: BTreeMap<u64, Segment>,
    name: String,
    write_offset: u64,
}

impl Topic {
    pub fn new(name: String) -> Self {
        if PathBuf::from(std::env::current_dir().unwrap().join(&name)).exists() {
            panic!("topic with name: {} already exist!", &name);
        }

        Self {
            name: name.clone(),
            segments: BTreeMap::from([(0, Segment::new(name.clone(), 0).unwrap())]),
            write_offset: 0,
        }
    }

    /// It finds the latest segment, and call its write method
    /// then update topic's global offset
    /// finally, rotate the segment when necessary
    /// # Arguments
    /// * message - the message being written to the  topic
    /// # Returns
    /// Result indicates the write is successful or not
    pub fn write(&mut self, message: &Message) -> io::Result<()> {
        if let Some(mut last_entry) = self.segments.last_entry() {
            let last_segment = last_entry.get_mut();

            last_segment.write(message)?;

            self.write_offset = last_segment.base_offset() + last_segment.write_position();

            if last_segment.write_position() >= Segment::SEGMENT_SIZE {
                self.segments.insert(
                    self.write_offset,
                    Segment::new(self.name.clone(), self.write_offset)?,
                );
            }

            Ok(())
        } else {
            panic!("Not able to find last segment from topic: {}.", self.name);
        }
    }

    /// It reads and returns the message with given offset
    /// # Arguments
    /// * offset - self explanatory
    /// # Returns the message
    pub fn read(&mut self, offset: u64) -> io::Result<Message> {
        let target_segment = match self.segments.range_mut(..=offset).next_back() {
            Some((_, segment)) => segment,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("cannot find corresponding segment per offset: {}", offset),
                ));
            }
        };

        let local_position = offset - target_segment.base_offset();
        target_segment.read(local_position)
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use rand::Rng;
    use rstest::fixture;
    use rstest::rstest;

    use crate::message::Message;
    use crate::topic::topic::Topic;

    struct TestTopic {
        topic: Topic,
    }

    /// Needs to use random charaters as test topic names
    /// to prevent concurrent issue that different test cases
    /// interacting with the same topic and the same segment file
    impl TestTopic {
        fn new() -> Self {
            let topic = Topic::new(String::from(generate_random_chars()));
            Self { topic }
        }
    }

    impl Drop for TestTopic {
        fn drop(&mut self) {
            let topic_path_buf = std::env::current_dir()
                .unwrap()
                .join("data")
                .join(&self.topic.name);
            fs::remove_dir_all(topic_path_buf).unwrap();
        }
    }

    // todo: use Deref to automatically ref to the inner topic
    #[fixture]
    fn test_topic() -> TestTopic {
        TestTopic::new()
    }

    #[rstest]
    fn test_write(mut test_topic: TestTopic) {
        let first_msg = Message::new(String::from("hello world!"));
        test_topic.topic.write(&first_msg).unwrap();
        let mut last_entry = test_topic.topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(0).unwrap();
        assert_eq!(&message.content, &first_msg.content);
        assert_eq!(test_topic.topic.write_offset, 24);

        let second_msg = Message::new(String::from("hello world again!"));
        test_topic.topic.write(&second_msg).unwrap();
        let mut last_entry = test_topic.topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(24).unwrap();
        assert_eq!(&message.content, &second_msg.content);

        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        test_topic.topic.write(&second_msg).unwrap();
        assert_eq!(test_topic.topic.segments.len(), 3);
    }

    #[rstest]
    fn test_read(mut test_topic: TestTopic) {
        assert_eq!(test_topic.topic.segments.len(), 1);

        let message = Message::new(String::from("testing_read"));
        test_topic.topic.write(&message).unwrap();

        assert_eq!(test_topic.topic.read(0).unwrap().content, message.content);
    }

    pub fn generate_random_chars() -> String {
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..26);
                (b'a' + idx) as char
            })
            .collect()
    }
}
