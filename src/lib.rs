pub mod client;
pub mod message;
pub mod storage;
pub mod topic;

#[cfg(test)]
pub mod test_utils {
    use rand::Rng;

    use crate::topic::topic::Topic;
    use std::fs;

    pub struct TestTopic {
        pub topic: Topic,
    }

    /// Needs to use random charaters as test topic names
    /// to prevent concurrent issue that different test cases
    /// interacting with the same topic and the same segment file
    impl TestTopic {
        pub fn new() -> Self {
            let topic = Topic::new(String::from(generate_random_chars()));
            Self { topic }
        }
    }

    impl Drop for TestTopic {
        fn drop(&mut self) {
            let topic_path_buf = std::env::current_dir()
                .unwrap()
                .join("data")
                .join(&self.topic.name());
            fs::remove_dir_all(topic_path_buf).unwrap();
        }
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
