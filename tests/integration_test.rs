use message_queue_rust::{client::consumer_group::ConsumerGroup, topic::topic::Topic};
use segment_rust::message::Message;

#[test]
fn test_read_write() {
    // as a producer, I would like to write a message to a topic
    // as a consumer, I would like to read a message from a topic
    let mut consumer_group = ConsumerGroup::new("integration_test_consumer_group");
    let first_topic =
        Topic::new("integration_test_first_Topic").expect("was not able to create the first topic");
    let first_message = Message::new("1st_message");
    consumer_group
        .write(first_topic.name(), &first_message.content)
        .expect("was not able to write 1st message to the first topic");
    let second_message = Message::new("2nd_message");
    consumer_group
        .write(first_topic.name(), &second_message.content)
        .expect("not able to write 2nd message to the first topic");

    let first_polled_msg = consumer_group
        .poll(first_topic.name())
        .expect("was not able to poll the first message");
    let second_polled_msg = consumer_group
        .poll(first_topic.name())
        .expect("was not able to poll the second message");

    assert_eq!(first_polled_msg.content, first_message.content);
    assert_eq!(second_polled_msg.content, second_message.content);
}
