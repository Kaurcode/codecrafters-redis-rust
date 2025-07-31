use std::cmp::max;
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
}

pub trait KeyValueStoreEntry: Send {
    fn get_value(&self) -> Result<&String, &'static str>;
    fn get_expiry(&self) -> &Option<SystemTime>;
    fn _push(&mut self, value: String) -> Result<usize, &'static str>;
    fn append(&mut self, other: &mut Vec<String>) -> Result<usize, &'static str>;
    fn prepend(&mut self, other: Vec<String>) -> Result<usize, &'static str>;
    fn pop_front(&mut self, amount: usize) -> Result<Vec<String>, &'static str>;
    fn get_subslice(&self, start: isize, end: isize) -> Result<Option<&[String]>, &'static str>;
    fn len(&self) -> Result<usize, &'static str>;
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
    fn _push(&mut self, _value: String) -> Result<usize, &'static str> {
        Err("String value, not list - pushing to a value is not allowed")
    }
    fn append(&mut self, _other: &mut Vec<String>) -> Result<usize, &'static str> {
        Err("String value, not list - appending to a value is not allowed")
    }
    fn prepend(&mut self, _other: Vec<String>) -> Result<usize, &'static str> {
        Err("String value, not list - prepending to a value is not allowed")
    }
    fn pop_front(&mut self, _amount: usize) -> Result<Vec<String>, &'static str> {
        Err("String value, not list - pop to a value is not allowed")
    }
    fn get_subslice(&self, _start: isize, _end: isize) -> Result<Option<&[String]>, &'static str> {
        Err("String value, not list - getting a subslice is not allowed")
    }
    fn len(&self) -> Result<usize, &'static str> {
        Ok(self.value.len())
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
    fn _push(&mut self, value: String) -> Result<usize, &'static str> {
        self.list.push(value);
        Ok(self.list.len())
    }
    fn append(&mut self, other: &mut Vec<String>) -> Result<usize, &'static str> {
        self.list.append(other);
        Ok(self.list.len())
    }
    fn prepend(&mut self, mut other: Vec<String>) -> Result<usize, &'static str> {
        other.append(&mut self.list);
        self.list = other;
        Ok(self.list.len())
    }
    fn pop_front(&mut self, mut amount: usize) -> Result<Vec<String>, &'static str> {
        amount = max(amount, self.list.len());
        Ok(self.list.drain(..amount).collect())
    }
    fn get_subslice(&self, start: isize, end: isize) -> Result<Option<&[String]>, &'static str> {
        let list_length: usize = self.list.len();

        let start: usize = normalize_index(start, list_length);
        let mut end: usize = normalize_index(end, list_length);

        if list_length <= end {
            end = list_length - 1;
        }

        Ok(self.list.get(start..=end))
    }
    fn len(&self) -> Result<usize, &'static str> {
        Ok(self.list.len())
    }
}

fn normalize_index(index: isize, list_length: usize) -> usize {
    if index < 0 {
        let normalized_index: usize = (-1 * index) as usize;
        if list_length < normalized_index {
            return 0;
        }
        return list_length - normalized_index;
    }
    index as usize
}
