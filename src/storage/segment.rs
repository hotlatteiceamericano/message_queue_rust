use crate::message::Message;
use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Read, Seek, Write},
    path::PathBuf,
};

pub struct Segment {
    base_offset: u64,
    write_position: u64,
    file: File,
}

impl Segment {
    pub fn new(base_offset: u64, path: PathBuf) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

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

    pub fn write(&mut self, message: &Message) -> io::Result<u64> {
        let serialized_msg = bincode::serialize(message)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let msg_len = serialized_msg.len() as u32;

        self.file.write_all(&msg_len.to_be_bytes())?;
        self.file.write_all(&serialized_msg)?;
        self.file.flush()?;

        self.write_position += 4 + msg_len as u64;

        Ok(self.base_offset + self.write_position)
    }

    /// #Arguments
    /// * `offset` - the local offset to this file
    /// it is expected for topic to find the local offset from a global offset
    /// # Returns
    /// message at the give offset
    pub fn read(&mut self, offset: u64) -> io::Result<Message> {
        // move the file cursor to the given offset
        // return the message
        self.file.seek(io::SeekFrom::Start(offset))?;

        let mut len_bytes = [0u8; 4];
        self.file.read_exact(&mut len_bytes)?;
        let msg_len = u32::from_be_bytes(len_bytes);

        let mut msg_bytes = vec![0u8; msg_len as usize];
        self.file.read_exact(&mut msg_bytes)?;

        bincode::deserialize::<Message>(&msg_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod test {

    use tempfile::NamedTempFile;

    use crate::storage::segment::Message;
    use crate::storage::segment::Segment;

    #[test]
    fn test_write() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut segment = Segment::new(0, temp_file.path().to_path_buf()).unwrap();
        let message = &Message::new(String::from("hello world!"));

        let latest_offset = segment.write(&message).unwrap();

        let serialized_msg = bincode::serialize(&message.content);
        assert_eq!(latest_offset, 4 + serialized_msg.unwrap().len() as u64);
    }

    #[test]
    pub fn test_read() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut segment = Segment::new(0, temp_file.path().to_path_buf()).unwrap();

        let message = Message::new(String::from("hello world!"));
        segment.write(&message).unwrap();

        let message_read = segment.read(0).unwrap();
        assert_eq!(message_read.content, "hello world!");
    }
}
