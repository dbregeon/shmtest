use std::hash::Hash;

mod pthread;
pub mod reader;
pub mod shm;
pub mod shm_syncer;
pub mod store_customer;
pub mod store_owner;
pub mod stream_consumer;
pub mod stream_producer;
pub mod writer;

pub struct ShmDefinition {
    name: String,
    size: usize,
}

impl ShmDefinition {
    pub fn new(name: String, size: usize) -> ShmDefinition {
        ShmDefinition { name, size }
    }
}

#[derive(Clone, Copy)]
pub struct TestRecord {
    pub value: (i32, i32),
}

impl Record<i32> for TestRecord {
    fn key(&self) -> i32 {
        println!("{:?}", self.value);
        self.value.0.clone()
    }
}

pub trait Key: Eq + Hash + Clone {}

pub trait Record<K>: Copy {
    fn key(&self) -> K;
}
