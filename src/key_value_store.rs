use std::collections::HashMap;
use std::time::SystemTime;


pub trait KeyValueStore: Send {
    fn insert(
        &mut self, 
        key: String, 
        entry: Box<dyn KeyValueStoreEntry>
    ) -> Option<Box<dyn KeyValueStoreEntry>>;
    
    fn get(&self, key: &String) -> Option<&Box<dyn KeyValueStoreEntry>>;
    fn get_mut(&mut self, key: &String) -> Option<&mut Box<dyn KeyValueStoreEntry>>;
    fn remove(&mut self, key: &String) -> Option<Box<dyn KeyValueStoreEntry>>;
    fn push(&mut self, key: String, value: String) -> Result<usize, &'static str>;
}

pub struct InMemoryKeyValueStore {
    store: HashMap<String, Box<dyn KeyValueStoreEntry>>,
}

impl InMemoryKeyValueStore {
    pub fn new() -> Self {
        InMemoryKeyValueStore {
            store: HashMap::new(),
        }
    }
}

impl KeyValueStore for InMemoryKeyValueStore {
    fn insert(
        &mut self, 
        key: String, 
        entry: Box<dyn KeyValueStoreEntry>
    ) -> Option<Box<dyn KeyValueStoreEntry>> {
        self.store.insert(key, entry)
    }
    
    fn get(&self, key: &String) -> Option<&Box<dyn KeyValueStoreEntry>> {
        self.store.get(key)
    }
    fn get_mut(&mut self, key: &String) -> Option<&mut Box<dyn KeyValueStoreEntry>> {
        self.store.get_mut(key)
    }
    fn remove(&mut self, key: &String) -> Option<Box<dyn KeyValueStoreEntry>> {
        self.store.remove(key)
    }
    fn push(&mut self, key: String, value: String) -> Result<usize, &'static str> {
        if let Some(entry) = self.get_mut(&key) {
            return entry.push(value);
        }
        
        let mut entry: KeyValueStoreListEntry = KeyValueStoreListEntry::new();
        let return_value: Result<usize, &str> = entry.push(value);
        self.insert(key, Box::new(entry));
        return_value
        
    }
}

pub trait KeyValueStoreEntry: Send {
    fn get_value(&self) -> Result<&String, &'static str>;
    fn get_expiry(&self) -> &Option<SystemTime>;
    fn push(&mut self, value: String) -> Result<usize, &'static str>;
}

pub struct KeyValueStoreStringEntry {
    pub value: String,
    pub expiry: Option<SystemTime>,
}

impl KeyValueStoreEntry for KeyValueStoreStringEntry {
    fn get_value(&self) -> Result<&String, &'static str> {
        Ok(&self.value)
    }
    fn get_expiry(&self) -> &Option<SystemTime> {
        &self.expiry
    }
    fn push(&mut self, _value: String) -> Result<usize, &'static str> {
        Err("String value, not list - appending to a value is not allowed")
    }
}

pub struct KeyValueStoreListEntry {
    list: Vec<String>,
    expiry: Option<SystemTime>,
}

impl KeyValueStoreListEntry {
    pub fn new() -> Self {
        KeyValueStoreListEntry {
            list: Vec::new(),
            expiry: None,
        }
    }
    
    pub fn _new_with_expiry(expiry: Option<SystemTime>) -> Self {
        KeyValueStoreListEntry {
            list: Vec::new(),
            expiry,
        }
    }
}

impl KeyValueStoreEntry for KeyValueStoreListEntry {
    fn get_value(&self) -> Result<&String, &'static str> {
        Err("Not yet implemented")
    }
    fn get_expiry(&self) -> &Option<SystemTime> {
        &self.expiry
    }
    fn push(&mut self, value: String) -> Result<usize, &'static str> {
        self.list.push(value);
        Ok(self.list.len())
    }
}
