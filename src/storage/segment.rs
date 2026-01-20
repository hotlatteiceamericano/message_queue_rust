use crate::message::Message;
use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Read, Seek, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub struct Segment {
    base_offset: u64,
    write_position: u64,
    file: File,
}

impl Segment {
    pub const SEGMENT_SIZE: u64 = 128;

    pub fn new(topic_name: &str, base_offset: u64) -> io::Result<Self> {
        let path = Self::create_path(topic_name.to_string(), base_offset)?;

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            base_offset,
            write_position: 0,
            file,
        })
    }

    pub fn base_offset(&self) -> u64 {
        self.base_offset
    }

    pub fn write_position(&self) -> u64 {
        self.write_position
    }

    /// # Arguments
    /// * `message` - the message being written to the segment
    /// # Returns
    /// new local write offset after written the given message
    pub fn write(&mut self, message: &Message) -> io::Result<u64> {
        let serialized_msg = bincode::serialize(message)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.file
            .write_all(&message.content_length().to_be_bytes())?;
        self.file.write_all(&serialized_msg)?;
        self.file.flush()?;

        self.write_position += message.total_length() as u64;

        Ok(self.write_position)
    }

    /// #Arguments
    /// * `offset` - the local offset to this file
    /// it is expected for topic to find the local offset from a global offset
    /// # Returns
    /// message at the give offset
    pub fn read(&mut self, offset: u64) -> io::Result<Message> {
        self.file.seek(io::SeekFrom::Start(offset))?;

        let mut len_bytes = [0u8; Message::MESSAGE_LENGTH as usize];
        self.file.read_exact(&mut len_bytes)?;
        let msg_len = u32::from_be_bytes(len_bytes);

        let mut msg_bytes = vec![0u8; msg_len as usize];
        self.file.read_exact(&mut msg_bytes)?;

        bincode::deserialize::<Message>(&msg_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn is_full(&self) -> bool {
        self.write_position() >= Segment::SEGMENT_SIZE
    }

    fn create_path(topic_name: String, base_offset: u64) -> io::Result<PathBuf> {
        let project_root = std::env::current_dir()?;
        let path = project_root
            .join("data")
            .join(topic_name)
            .join(format!("{:08}", base_offset))
            .with_extension("queue");
        eprintln!("created path name: {}", path.to_str().unwrap());
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        Ok(path)
    }
}

#[cfg(test)]
mod test {

    use std::fs;

    use rand::Rng;
    use rstest::fixture;
    use rstest::rstest;

    use crate::storage::segment::Message;
    use crate::storage::segment::Segment;

    #[fixture]
    fn random_topic_name() -> String {
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..26);
                (b'a' + idx) as char
            })
            .collect()
    }

    #[rstest]
    fn test_write(random_topic_name: String) {
        let mut segment = Segment::new(&random_topic_name, 0).unwrap();
        let message = &Message::new("hello world!");

        let latest_offset = segment.write(&message).unwrap();

        let serialized_msg = bincode::serialize(&message.content);
        assert_eq!(latest_offset, 4 + serialized_msg.unwrap().len() as u64);

        remove_path(random_topic_name.as_str());
    }

    #[rstest]
    pub fn test_read(random_topic_name: String) {
        let mut segment = Segment::new(&random_topic_name, 0).unwrap();

        let message = Message::new("hello world!");
        segment.write(&message).unwrap();

        let message_read = segment
            .read(0)
            .unwrap_or_else(|e| panic!("error when read from the segment: {:#?}", e));
        assert_eq!(message_read.content, "hello world!");

        remove_path(random_topic_name.as_str());
    }

    fn remove_path(topic_name: &str) {
        let topic_path_buf = std::env::current_dir()
            .unwrap()
            .join("data")
            .join(topic_name);
        fs::remove_dir_all(topic_path_buf).unwrap();
    }
}
