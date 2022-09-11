use nix::Result;

use crate::common::shm::ShmMap;
use crate::common::ShmDefinition;

use super::shm_syncer::ShmSync;

pub struct ShmStream<E: Copy> {
    map: ShmMap,
    syncer: ShmSync<ShmMap>,
    sequence_number: *const u64,
    end_ptr: *const E,
    next_sequence: u64,
}

impl<E: Copy> ShmStream<E> {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        let name = definition.name.clone();
        ShmSync::<ShmMap>::load(name).and_then(|syncer| {
            ShmMap::open(definition).map(|m| {
                // We keep the number of written bytes of the beginning
                let sequence_number = m.start_ptr() as *const u64;
                // Ensure Alignment
                let end_ptr = unsafe { (m.start_ptr() as *const E).add(1) };
                Self {
                    map: m,
                    syncer: syncer,
                    sequence_number: sequence_number,
                    end_ptr: end_ptr,
                    next_sequence: 1,
                }
            })
        })
    }

    pub fn next(&mut self) -> Option<E> {
        let current_sequence = unsafe { self.sequence_number.read_volatile() };
        if current_sequence < self.next_sequence {
            self.syncer.wait()
        } else {
            Ok(())
        }
        .ok()
        .and_then(|_| {
            if current_sequence >= self.next_sequence {
                let record = unsafe { self.end_ptr.read_volatile() };
                self.next_sequence += 1;

                unsafe {
                    self.end_ptr = self.end_ptr.add(1);
                }
                Some(record)
            } else {
                None
            }
        })
    }
}
