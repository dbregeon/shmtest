extern crate shmtest;

use shmtest::common::store_owner::ShmStore;
use shmtest::common::writer::ShmWriter;
use shmtest::common::{ShmDefinition, TestRecord};

use std::io::Write;

fn main() {
    let writer_definition = ShmDefinition::new("test_writer".to_string(), 10);
    let mut writer = ShmWriter::open(writer_definition).unwrap();

    let store_definition = ShmDefinition::new("test_store".to_string(), 1024);
    let mut store: ShmStore<i32, TestRecord> = ShmStore::open(store_definition).unwrap();

    writer.write("test1".as_bytes()).unwrap();
    writer.write("test2".as_bytes()).unwrap();
    writer.flush().unwrap();

    store.put(TestRecord { value: (1, 11) }).unwrap();

    store.put(TestRecord { value: (2, 12) }).unwrap();

    store.put(TestRecord { value: (3, 13) }).unwrap();

    store.put(TestRecord { value: (1, 21) }).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(60));

    store.put(TestRecord { value: (4, 14) }).unwrap();

    writer.close().unwrap();
    store.close().unwrap()
}
