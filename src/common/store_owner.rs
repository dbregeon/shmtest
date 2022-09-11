use std::collections::HashMap;
use std::hash::Hash;
use std::mem::size_of;

use nix::errno::Errno;
use nix::Result;

use crate::common::shm::MutableShmMap;
use crate::common::{Record, ShmDefinition};

pub struct ShmStore<K, R: Record<K>> {
    map: MutableShmMap,
    written_records_ptr: *mut usize,
    end_ptr: *mut R,
    available: usize,
    index: HashMap<K, usize>,
}

impl<K: Eq + Hash, R: Record<K>> ShmStore<K, R> {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        let size = definition.size;
        MutableShmMap::create(definition).map(|m| {
            // We keep the number of written bytes of the beginning
            let written_records_ptr = m.start_ptr() as *mut usize;
            // Ensure Alignment
            let end_ptr = unsafe { (m.start_ptr() as *mut R).add(1) };
            unsafe { *written_records_ptr = 0 };
            Self {
                map: m,
                written_records_ptr: written_records_ptr,
                end_ptr: end_ptr,
                available: (size - size_of::<u8>()) / size_of::<R>(),
                index: HashMap::new(),
            }
        })
    }

    pub fn put(&mut self, record: R) -> Result<()> {
        if self.available > 0 {
            let key = record.key();
            let written_records = unsafe { self.written_records_ptr.read_volatile() };
            match self.index.get(&key) {
                Some(i) => {
                    unsafe {
                        self.end_ptr
                            .sub(written_records - i + 1)
                            .write_volatile(record)
                    };
                }
                None => {
                    unsafe {
                        self.end_ptr.write(record);
                        self.written_records_ptr.write_volatile(written_records + 1);
                        self.end_ptr = self.end_ptr.add(1);
                        self.index.insert(key, *self.written_records_ptr as usize);
                    };
                    self.available -= 1;
                }
            };
            Ok(())
        } else {
            Err(Errno::ENOMEM)
        }
    }
}
