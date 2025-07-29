use std::time::SystemTime;

pub struct KeyValueStoreEntry {
    pub value: String,
    pub expiry: Option<SystemTime>,
}
