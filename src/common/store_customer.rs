use std::collections::HashMap;
use std::hash::Hash;

use nix::errno::Errno;
use nix::Result;

use crate::common::shm::ShmMap;
use crate::common::{Record, ShmDefinition};

pub struct ShmStore<K, R: Record<K>> {
    map: ShmMap,
    written_records_ptr: *const usize,
    end_ptr: *const R,
    next_read: usize,
    index: HashMap<K, usize>,
}

impl<K: Eq + Hash + Clone, R: Record<K>> ShmStore<K, R> {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        ShmMap::open(definition).map(|m| {
            // We keep the number of written bytes of the beginning
            let written_records_ptr = m.start_ptr() as *const usize;
            // Ensure Alignment
            let end_ptr = unsafe { (m.start_ptr() as *const R).add(1) };
            Self {
                map: m,
                written_records_ptr: written_records_ptr,
                end_ptr: end_ptr,
                next_read: 0,
                index: HashMap::new(),
            }
        })
    }

    pub fn get(&mut self, key: &K) -> Result<R> {
        let records_count = unsafe { self.written_records_ptr.read_volatile() };
        if !self.index.contains_key(key) {
            while self.next_read < records_count {
                let record = unsafe { self.end_ptr.read_volatile() };
                self.index.insert(record.key().clone(), self.next_read);
                self.next_read += 1;

                unsafe {
                    self.end_ptr = self.end_ptr.add(1);
                }
            }
        };
        self.index
            .get(key)
            .ok_or(Errno::ENOKEY)
            .and_then(|i| unsafe { Ok(self.end_ptr.sub(records_count - i).read_volatile()) })
    }
}
