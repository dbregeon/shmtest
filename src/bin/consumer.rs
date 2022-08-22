extern crate shmtest;

use shmtest::common::reader::ShmReader;
use shmtest::common::store_customer::ShmStore;
use shmtest::common::{ShmDefinition, TestRecord};
use std::io::Read;

fn main() {
    test_reader();
    test_store_client();
}

fn test_reader() {
    let definition = ShmDefinition::new("test_writer".to_string(), 10);
    let mut reader = ShmReader::open(definition).unwrap();
    let mut buffer = vec![0 as u8; 1024];

    reader.read(&mut buffer).unwrap();
    reader.close().unwrap();

    println!("{}", std::str::from_utf8(&buffer).unwrap());
}

fn test_store_client() {
    let definition = ShmDefinition::new("test_store".to_string(), 1024);
    let mut store: ShmStore<i32, TestRecord> = ShmStore::open(definition).unwrap();

    println!("Found {:?}", store.get(&1).unwrap().value);
    println!("Found {:?}", store.get(&2).unwrap().value);
    println!("Found {:?}", store.get(&3).unwrap().value);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        match store.get(&4) {
            Ok(r) => {
                println!("Found {:?}", r.value);
                break;
            }
            Err(e) => {}
        }
    }
}
