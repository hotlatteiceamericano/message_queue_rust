use crate::message::Message;
use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Write},
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
}
