use revm::primitives::U256;

use crate::test::case::input::value::Value;

pub fn revm_bytes_to_vec_value(bytes: revm::primitives::Bytes) -> Vec<Value> {
    let mut datas = vec![];
    datas.extend_from_slice(&bytes);
    let mut data_value = vec![];
    for data in datas.chunks(32) {
        let mut value = [0u8; 32];
        value[..data.len()].copy_from_slice(data);
        data_value.push(Value::Known(U256::from_be_bytes(value)));
    }
    data_value
}

pub fn revm_topics_to_vec_value(revm_topics: &[revm::primitives::B256]) -> Vec<Value> {
    let mut topics = vec![];
    for topic in revm_topics.iter() {
        let mut topic_value = [0u8; 32];
        topic_value.copy_from_slice(topic.as_slice());
        topics.push(Value::Known(U256::from_be_bytes(topic_value)));
    }
    topics
}
