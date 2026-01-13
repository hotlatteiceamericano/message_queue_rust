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

    pub fn write(&mut self, message: &Message) -> io::Result<()> {
        if let Some(mut last_segment) = self.segments.last_entry() {
            last_segment.get_mut().write(message)?;
            self.write_offset =
                last_segment.get().base_offset() + last_segment.get().write_position();
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
    use tempfile::NamedTempFile;

    use crate::message::Message;
    use crate::storage::segment::Segment;
    use crate::topic::topic::Topic;

    #[fixture]
    fn test_topic() -> Topic {
        let temp_file = NamedTempFile::new().unwrap();
        let mut topic = Topic::new(String::from("test topic"));
        topic
            .segments
            .insert(0, Segment::new(0, temp_file.path().to_path_buf()).unwrap());
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
