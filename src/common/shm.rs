use std::os::unix::io::RawFd;
use std::ptr::null_mut;

use nix::fcntl::OFlag;
use nix::sys::mman::{mmap, munmap, shm_open, shm_unlink, MapFlags, ProtFlags};
use nix::sys::stat::Mode;
use nix::unistd::{close, ftruncate};
use nix::Result;

use libc::c_void;

use crate::common::ShmDefinition;

pub struct ShmMap {
    definition: ShmDefinition,
    map_ptr: *const u8,
}

pub struct MutableShmMap {
    definition: ShmDefinition,
    map_ptr: *mut u8,
}

impl MutableShmMap {
    pub fn create(definition: ShmDefinition) -> Result<Self> {
        shm_open(
            definition.name.as_str(),
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR, //create exclusively (error if collision) and write to allow resize
            Mode::S_IRUSR | Mode::S_IWUSR,                  //Permission allow user+rw
        )
        .and_then(|fd| {
            create_mmap(&definition, fd, ProtFlags::PROT_WRITE).and_then(|p| {
                close(fd).and_then(|_| {
                    Ok(Self {
                        definition,
                        map_ptr: p as *mut u8,
                    })
                })
            })
        })
    }

    pub fn delete(self) -> Result<()> {
        unsafe { munmap(self.map_ptr as *mut _, self.definition.size) }
            .and_then(|_| shm_unlink(self.definition.name.as_str()))
    }

    pub fn start_ptr(&self) -> *mut u8 {
        self.map_ptr as *mut u8
    }

    pub fn offset(&self, count: usize) -> *mut u8 {
        unsafe { self.map_ptr.add(count) }
    }
}

fn create_mmap(definition: &ShmDefinition, fd: RawFd, flags: ProtFlags) -> Result<*mut c_void> {
    match ftruncate(fd, definition.size as _) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    unsafe {
        mmap(
            null_mut(),           //Desired addr
            definition.size,      //size of mapping
            flags,                //Permissions on pages
            MapFlags::MAP_SHARED, //What kind of mapping
            fd,                   //fd
            0,                    //Offset into fd
        )
    }
}

impl ShmMap {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        shm_open(
            definition.name.as_str(),
            OFlag::O_RDWR, // write to allow resize
            Mode::S_IRUSR, //Permission allow user+rw
        )
        .and_then(|fd| {
            create_mmap(&definition, fd, ProtFlags::PROT_READ).and_then(|p| {
                close(fd).and_then(|_| {
                    Ok(Self {
                        definition,
                        map_ptr: p as *const u8,
                    })
                })
            })
        })
    }

    pub fn start_ptr(&self) -> *const u8 {
        self.map_ptr
    }

    pub fn offset(&self, count: usize) -> *const u8 {
        unsafe { self.map_ptr.add(count) }
    }

    pub fn close(self) -> Result<()> {
        unsafe { munmap(self.map_ptr as *mut _, self.definition.size) }
    }
}
