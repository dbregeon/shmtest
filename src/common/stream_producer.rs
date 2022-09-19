use std::mem::size_of;

use nix::errno::Errno;
use nix::Result;

use crate::common::shm::MutableShmMap;
use crate::common::ShmDefinition;

use super::shm_syncer::ShmSync;

pub struct ShmStream<E: Copy> {
    map: MutableShmMap,
    syncer: ShmSync<MutableShmMap>,
    sequence_number: *mut u64,
    end_ptr: *mut E,
    available: usize,
}

impl<E: Copy> ShmStream<E> {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        let size = definition.size;
        let name = definition.name.clone();
        ShmSync::<MutableShmMap>::create(name).and_then(|syncer| {
            MutableShmMap::create(definition).map(|m| {
                // We keep the number of written bytes of the beginning
                let sequence_number = m.start_ptr() as *mut u64;
                // Ensure Alignment
                let end_ptr = unsafe { (m.start_ptr() as *mut E).add(1) };
                unsafe { *sequence_number = 0 };
                Self {
                    map: m,
                    syncer: syncer,
                    sequence_number: sequence_number,
                    end_ptr: end_ptr,
                    available: (size - size_of::<u64>()) / size_of::<E>(),
                }
            })
        })
    }

    pub fn insert(&mut self, event: E) -> Result<()> {
        if self.available > 0 {
            let sequence_number = unsafe { self.sequence_number.read_volatile() };
            unsafe {
                self.end_ptr.write(event);
                self.sequence_number.write_volatile(sequence_number + 1);
                self.end_ptr = self.end_ptr.add(1);
            };
            self.syncer.notify_all();
            self.available -= 1;
            Ok(())
        } else {
            Err(Errno::ENOMEM)
        }
    }
}
