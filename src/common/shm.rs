use std::os::unix::io::RawFd;
use std::ptr::null_mut;

use log::debug;
use nix::fcntl::OFlag;
use nix::sys::mman::{mmap, munmap, shm_open, shm_unlink, MapFlags, ProtFlags};
use nix::sys::stat::Mode;
use nix::unistd::{close, ftruncate};
use nix::Result;

use libc::c_void;

use crate::common::ShmDefinition;

pub struct ShmMap {
    definition: ShmDefinition,
    start_ptr: *const u8,
}

pub struct MutableShmMap {
    definition: ShmDefinition,
    start_ptr: *const u8,
}

impl Drop for MutableShmMap {
    fn drop(&mut self) {
        debug!("dropping mutableshm {}", self.definition.name);
        unsafe { munmap(self.start_ptr as *mut _, self.definition.size) }
            .and_then(|_| shm_unlink(self.definition.name.as_str()))
            .unwrap();
    }
}

impl MutableShmMap {
    pub fn create(definition: ShmDefinition) -> Result<Self> {
        shm_open(
            definition.name.as_str(),
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR, //create exclusively (error if collision) and write to allow resize
            Mode::S_IRUSR | Mode::S_IWUSR,                  //Permission allow user+rw
        )
        .and_then(|fd| {
            create_mmap(
                &definition,
                fd,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            )
            .and_then(|p| {
                close(fd).and_then(|_| {
                    debug!("created mutableshm {}", definition.name);
                    Ok(Self {
                        definition,
                        start_ptr: p as *const u8,
                    })
                })
            })
        })
    }

    pub fn start_ptr(&self) -> *mut u8 {
        self.start_ptr as *mut u8
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

impl Drop for ShmMap {
    fn drop(&mut self) {
        debug!("dropping shm {}", self.definition.name);
        unsafe { munmap(self.start_ptr as *mut _, self.definition.size).unwrap() }
    }
}

impl ShmMap {
    pub fn open(definition: ShmDefinition) -> Result<Self> {
        shm_open(
            definition.name.as_str(),
            OFlag::O_RDWR,                 // write to allow resize
            Mode::S_IRUSR | Mode::S_IWUSR, //Permission allow user+rw
        )
        .and_then(|fd| {
            create_mmap(
                &definition,
                fd,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            )
            .and_then(|p| {
                close(fd).and_then(|_| {
                    debug!("opened shm {}", definition.name);
                    Ok(Self {
                        definition,
                        start_ptr: p as *const u8,
                    })
                })
            })
        })
    }

    pub fn start_ptr(&self) -> *const u8 {
        self.start_ptr
    }
}
