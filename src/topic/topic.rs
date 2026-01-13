use std::{collections::BTreeMap, io};

use crate::{message::Message, storage::segment::Segment};

pub struct Topic {
    segments: BTreeMap<u64, Segment>,
    name: String,
    write_offset: u64,
}

impl Topic {
    pub fn new(name: String) -> Self {
        Self {
            name,
            segments: BTreeMap::new(),
            write_offset: 0,
        }
    }

    /// It fids the latest segment, and call its write method
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
}

#[cfg(test)]
mod test {
    use rstest::fixture;
    use rstest::rstest;

    use crate::message::Message;
    use crate::storage::segment::Segment;
    use crate::topic::topic::Topic;

    #[fixture]
    fn test_topic() -> Topic {
        let mut topic = Topic::new(String::from("test topic"));
        topic.segments.insert(
            0,
            Segment::new(String::from("the_segment_to_test_topic"), 0).unwrap(),
        );
        topic
    }

    #[rstest]
    fn test_write(mut test_topic: Topic) {
        let first_msg = Message::new(String::from("hello world!"));
        test_topic.write(&first_msg).unwrap();
        let mut last_entry = test_topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(0).unwrap();
        assert_eq!(&message.content, &first_msg.content);
        assert_eq!(test_topic.write_offset, 24);

        let second_msg = Message::new(String::from("hello world again!"));
        test_topic.write(&second_msg).unwrap();
        let mut last_entry = test_topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(24).unwrap();
        assert_eq!(&message.content, &second_msg.content);
    }
}
