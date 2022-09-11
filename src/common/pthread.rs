use std::{
    mem::MaybeUninit,
    sync::{Condvar, Mutex, MutexGuard},
    time::Duration,
};

use log::debug;

use super::shm::{MutableShmMap, ShmMap};

pub struct ShmMutex<T> {
    shm: T,
    ptr: *mut Mutex<u8>,
}

impl ShmMutex<MutableShmMap> {
    pub fn init_in_shm(shm: MutableShmMap) -> Self {
        unsafe {
            let mutex = Mutex::new(0 as u8);
            let ptr = shm.start_ptr() as *mut Mutex<u8>;
            ptr.write(mutex);
            debug!("created mutex at {:?}", shm.start_ptr());
            ShmMutex::<MutableShmMap> { shm: shm, ptr: ptr }
        }
    }
}

impl<T> ShmMutex<T> {
    pub fn lock(&mut self) -> MutexGuard<u8> {
        unsafe {
            let guard = (*self.ptr).lock().unwrap();
            debug!("locked mutex at {:?}", self.ptr);
            guard
        }
    }
}

impl ShmMutex<ShmMap> {
    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Mutex<u8>;
        debug!("initialized mutex at {:?}", ptr);
        ShmMutex { shm: shm, ptr: ptr }
    }
}

pub struct ShmCondition<T> {
    shm: T,
    ptr: *mut Condvar,
}

impl ShmCondition<MutableShmMap> {
    pub fn init_in_shm(shm: MutableShmMap) -> Self {
        let condvar = Condvar::new();
        let ptr = shm.start_ptr() as *mut Condvar;
        unsafe {
            ptr.write(condvar);
            debug!("created cond at {:?}", shm.start_ptr());
            ShmCondition { shm: shm, ptr: ptr }
        }
    }

    pub fn notify_all(&mut self) {
        unsafe {
            (*self.ptr).notify_all();
            debug!("notified cond at {:?}", self.ptr);
        }
    }
}

impl ShmCondition<ShmMap> {
    pub fn wait<T>(&mut self, mutex: &mut ShmMutex<T>) {
        let guard = mutex.lock();
        unsafe {
            (*self.ptr)
                .wait_timeout(guard, Duration::from_secs(30))
                .unwrap();
            debug!("woke on condition at {:?}", self.ptr);
        }
    }

    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Condvar;
        debug!("initialized cond at {:?}", ptr);
        ShmCondition { shm: shm, ptr: ptr }
    }
}

pub struct ConditionAttr {
    ptr: *mut libc::pthread_condattr_t,
}

impl Drop for ConditionAttr {
    fn drop(&mut self) {
        debug!("destroying condattr");
        unsafe {
            libc::pthread_condattr_destroy(self.ptr);
        }
    }
}

impl ConditionAttr {
    pub fn init() -> nix::Result<Self> {
        unsafe {
            let cond_attr: *mut libc::pthread_condattr_t = MaybeUninit::uninit().as_mut_ptr();
            let result = libc::pthread_condattr_init(cond_attr);
            if 0 == result {
                debug!("initialized condattr");
                Ok(ConditionAttr { ptr: cond_attr })
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }

    pub fn set_pshared(&self) -> nix::Result<()> {
        unsafe {
            let result = libc::pthread_condattr_setpshared(self.ptr, libc::PTHREAD_PROCESS_SHARED);
            if 0 == result {
                debug!("shared condattr");
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use libc::pthread_mutex_t;

    use crate::common::{shm::MutableShmMap, ShmDefinition};

    use super::ShmMutex;

    #[test]
    fn init_mutex_produces_a_valid_mutex_from_shared_memory() {
        let definition =
            ShmDefinition::new("sync".to_string(), std::mem::size_of::<pthread_mutex_t>());
        let shm = MutableShmMap::create(definition).unwrap();
        let mut mutex = ShmMutex::init_in_shm(shm);

        mutex.lock();
    }
}
