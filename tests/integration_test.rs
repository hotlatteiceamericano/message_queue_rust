use message_queue_rust::{client::consumer_group::ConsumerGroup, topic::topic::Topic};

#[test]
fn test_read_write() {
    // as a producer, I would like to write a message to a topic
    // as a consumer, I would like to read a message from a topic
    let consumer_group = ConsumerGroup::new("integration_test_consumer_group");

    let first_topic = Topic::new("integration_test_first_Topic");
}
