use std::io::Read;

use nix::Result;

use crate::common::shm::ShmMap;
use crate::common::ShmDefinition;

pub struct ShmReader {
    map: ShmMap,
    written_bytes_ptr: *const u8,
    last_read_ptr: *const u8,
    read: usize,
}

impl ShmReader {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        ShmMap::open(definition).map(|m| {
            // We keep the number of written bytes of the beginning
            let written_bytes_ptr = m.start_ptr();
            let last_read_ptr = unsafe { written_bytes_ptr.add(1) };
            Self {
                map: m,
                written_bytes_ptr: written_bytes_ptr,
                last_read_ptr: last_read_ptr,
                read: 0,
            }
        })
    }
}

impl Read for ShmReader {
    fn read(&mut self, out: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        let readable_size = out
            .len()
            .min(unsafe { *self.written_bytes_ptr as usize } - self.read);
        if readable_size > 0 {
            unsafe {
                self.last_read_ptr.copy_to(out.as_mut_ptr(), readable_size);
                self.last_read_ptr = self.last_read_ptr.add(readable_size);
            }
            self.read = self.read + readable_size;
            Ok(readable_size)
        } else {
            Ok(0)
        }
    }
}
