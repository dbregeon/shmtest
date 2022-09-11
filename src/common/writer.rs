use std::io::Write;
use std::mem::size_of;

use nix::Result;

use crate::common::shm::MutableShmMap;
use crate::common::ShmDefinition;

pub struct ShmWriter {
    map: MutableShmMap,
    written_bytes_ptr: *mut u8,
    end_ptr: *mut u8,
    available: usize,
}

impl ShmWriter {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        let size = definition.size;
        MutableShmMap::create(definition).map(|m| {
            // We keep the number of written bytes of the beginning
            let written_bytes_ptr = m.start_ptr();
            let end_ptr = unsafe { written_bytes_ptr.add(1) };
            unsafe { *written_bytes_ptr = 0 };
            Self {
                map: m,
                written_bytes_ptr: written_bytes_ptr,
                end_ptr: end_ptr,
                available: size - size_of::<u8>(),
            }
        })
    }
}

impl Write for ShmWriter {
    fn write(&mut self, value: &[u8]) -> std::result::Result<usize, std::io::Error> {
        let writable_size = self.available.min(value.len());
        if writable_size > 0 {
            unsafe {
                self.end_ptr.copy_from(value.as_ptr(), writable_size);
                self.end_ptr = self.end_ptr.add(writable_size);
                *self.written_bytes_ptr += writable_size as u8;
            }
            self.available = self.available - writable_size;
            Ok(writable_size)
        } else {
            Ok(0)
        }
    }

    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}
