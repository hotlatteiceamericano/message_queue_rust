use std::{
    collections::BTreeMap,
    fs,
    hash::Hash,
    io::{self},
    path::PathBuf,
};

use anyhow::bail;
use segment_rust::{message::Message, segment, segment::Segment};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Topic {
    #[serde(skip)]
    segments: BTreeMap<u64, Segment<Message>>,
    name: String,
    write_offset: u64,
}

impl Topic {
    const SEGMENT_LENGTH_PER_TOPIC: u64 = 128;

    pub fn new(name: &str) -> anyhow::Result<Self> {
        if PathBuf::from(std::env::current_dir().unwrap().join(&name)).exists() {
            panic!("topic with name: {} already exist!", &name);
        }

        let first_segment = Segment::new(&Self::get_directory(name), 0)?;
        let topic = Self {
            name: name.to_string(),
            segments: BTreeMap::from([(0, first_segment)]),
            write_offset: 0,
        };

        topic.save()?;

        Ok(topic)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Loads a topic with given name
    /// # Arguments
    /// * topic_name - self explanatory
    pub fn load(topic_name: &str) -> anyhow::Result<Self> {
        let topic_file_path = Self::get_directory(topic_name);
        if topic_file_path.exists() {
            let segments = Self::load_segments(topic_name)?;
            let json = fs::read_to_string(topic_file_path.join("index.json"))?;
            let mut topic = serde_json::from_str::<Topic>(&json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            topic.segments = segments;

            Ok(topic)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Did not found topic with topic name: {}", &topic_name),
            )
            .into())
        }
    }

    /// Saves this instance to a json file
    /// So that the from function may find the metadata and loads it
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self)?;
        let path = std::env::current_dir()?
            .join("data")
            .join(&self.name)
            .join("index")
            .with_extension("json");
        fs::write(path, json)?;
        Ok(())
    }

    /// It finds the latest segment, and call its write method
    /// then update topic's global offset
    /// finally, rotate the segment when necessary
    /// # Arguments
    /// * message - the message being written to the  topic
    /// # Returns
    /// Result indicates the write is successful or not
    pub fn write(&mut self, message: &Message) -> anyhow::Result<()> {
        if let Some(mut last_entry) = self.segments.last_entry() {
            let last_segment = last_entry.get_mut();

            last_segment.write(message)?;

            self.write_offset = last_segment.base_offset() + last_segment.write_position();

            if last_segment.write_position() >= Self::SEGMENT_LENGTH_PER_TOPIC {
                self.segments.insert(
                    self.write_offset,
                    Segment::new(&Self::get_directory(&self.name), self.write_offset)?,
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
    pub fn read(&mut self, offset: &u64) -> anyhow::Result<Message> {
        let target_segment = match self.segments.range_mut(..=offset).next_back() {
            Some((_, segment)) => segment,
            None => {
                bail!("cannot find corresponding segment per offset: {}", offset);
            }
        };

        let local_position = offset - target_segment.base_offset();
        target_segment.read(local_position)
    }

    fn load_segments(topic_name: &str) -> anyhow::Result<BTreeMap<u64, Segment<Message>>> {
        let topic_directory = Self::get_directory(topic_name);
        let segment_file_paths = fs::read_dir(&topic_directory)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                path.extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .eq_ignore_ascii_case(segment::FILE_EXTENSION)
            })
            .collect::<Vec<PathBuf>>();

        let mut segments = BTreeMap::new();
        for segment_file_path in segment_file_paths {
            let base_offset = segment_file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Invalid segment filename")
                })?;
            let segment = Segment::new(&topic_directory, base_offset)?;
            segments.insert(base_offset, segment);
        }

        Ok(segments)
    }

    fn get_directory(topic_name: &str) -> PathBuf {
        std::env::current_dir()
            .expect("cannot load current directory")
            .join("data")
            .join(topic_name)
    }
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Topic {}

impl Hash for Topic {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[cfg(test)]
mod test {

    use rstest::fixture;
    use rstest::rstest;
    use segment_rust::message::Message;

    use crate::test_utils::TestTopic;
    use crate::topic::topic::Topic;

    // todo: use Deref to automatically ref to the inner topic
    #[fixture]
    fn test_topic() -> TestTopic {
        TestTopic::new()
    }

    #[rstest]
    fn test_write(mut test_topic: TestTopic) {
        let first_msg = Message::new("hello world!");
        test_topic.topic.write(&first_msg).unwrap();
        let mut last_entry = test_topic.topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(0).unwrap();
        assert_eq!(&message.content, &first_msg.content);
        assert_eq!(test_topic.topic.write_offset, 24);

        let second_msg = Message::new("hello world again!");
        test_topic.topic.write(&second_msg).unwrap();
        let mut last_entry = test_topic.topic.segments.last_entry().unwrap();
        let segment = last_entry.get_mut();
        let message = segment.read(24).unwrap();
        assert_eq!(&message.content, &second_msg.content);

        for _ in 0..8 {
            test_topic.topic.write(&second_msg).unwrap();
        }
        assert_eq!(test_topic.topic.segments.len(), 3);
    }

    #[rstest]
    fn test_read(mut test_topic: TestTopic) {
        assert_eq!(test_topic.topic.segments.len(), 1);

        let message = Message::new("testing_read");
        test_topic.topic.write(&message).unwrap();

        assert_eq!(test_topic.topic.read(&0).unwrap().content, message.content);
    }

    #[rstest]
    pub fn test_save_load(mut test_topic: TestTopic) {
        let message = Message::new("testing_for_save_and_load");
        test_topic.topic.write(&message).unwrap();

        test_topic.topic.save().unwrap();

        let topic_name = test_topic.topic.name();
        let mut loaded_topic = Topic::load(topic_name).unwrap();

        assert_eq!(loaded_topic.name, test_topic.topic.name);
        assert_eq!(loaded_topic.segments.len(), 1);
        assert_eq!(loaded_topic.read(&0).unwrap().content, message.content);
    }
}
