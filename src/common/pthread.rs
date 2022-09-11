use std::{
    mem::{ManuallyDrop, MaybeUninit},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use libc::{pthread_cond_t, pthread_mutex_t, timespec};
use log::debug;

use super::shm::{MutableShmMap, ShmMap};

pub struct ShmMutex<T> {
    should_destroy: bool,
    shm: ManuallyDrop<T>,
    ptr: *mut libc::pthread_mutex_t,
}

impl<T> Drop for ShmMutex<T> {
    fn drop(&mut self) {
        if self.should_destroy {
            debug!("destroying mutex");
            unsafe {
                libc::pthread_mutex_destroy(self.ptr);
            }
        }
        unsafe {
            ManuallyDrop::drop(&mut self.shm);
        }
    }
}

impl ShmMutex<MutableShmMap> {
    pub fn init_from_shm(shm: MutableShmMap, attr: MutexAttr) -> nix::Result<Self> {
        unsafe {
            let ptr = shm.start_ptr() as *mut pthread_mutex_t;
            let result = libc::pthread_mutex_init(ptr, attr.ptr);
            if 0 == result {
                debug!("created mutex at {:?}", shm.start_ptr());
                Ok(ShmMutex::<MutableShmMap> {
                    shm: ManuallyDrop::new(shm),
                    should_destroy: true,
                    ptr: ptr,
                })
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }
}

impl<T> ShmMutex<T> {
    pub fn lock(&mut self) -> nix::Result<()> {
        unsafe {
            let result = libc::pthread_mutex_lock(self.ptr);
            if 0 == result {
                debug!("locked mutex at {:?}", self.ptr);
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }

    pub fn unlock(&mut self) -> nix::Result<()> {
        unsafe {
            let result = libc::pthread_mutex_unlock(self.ptr);
            if 0 == result {
                debug!("unlocked mutex at {:?}", self.ptr);
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }
}

impl ShmMutex<ShmMap> {
    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut pthread_mutex_t;
        debug!("initialized mutex at {:?}", ptr);
        ShmMutex {
            should_destroy: false,
            shm: ManuallyDrop::new(shm),
            ptr: ptr,
        }
    }
}

pub struct MutexAttr {
    ptr: *mut libc::pthread_mutexattr_t,
}

impl Drop for MutexAttr {
    fn drop(&mut self) {
        debug!("destroying nutexattr");
        unsafe {
            libc::pthread_mutexattr_destroy(self.ptr);
        }
    }
}

impl MutexAttr {
    pub fn init() -> nix::Result<Self> {
        unsafe {
            let mutex_attr: *mut libc::pthread_mutexattr_t = MaybeUninit::uninit().as_mut_ptr();
            let result = libc::pthread_mutexattr_init(mutex_attr);
            if 0 == result {
                debug!("initialized nutexattr");
                Ok(MutexAttr { ptr: mutex_attr })
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }

    pub fn set_pshared(&self) -> nix::Result<()> {
        unsafe {
            let result = libc::pthread_mutexattr_setpshared(self.ptr, libc::PTHREAD_PROCESS_SHARED);
            if 0 == result {
                debug!("shared nutexattr");
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }
}

pub struct ShmCondition<T> {
    should_destroy: bool,
    shm: ManuallyDrop<T>,
    ptr: *mut libc::pthread_cond_t,
}

impl<T> Drop for ShmCondition<T> {
    fn drop(&mut self) {
        if self.should_destroy {
            debug!("destroying cond");
            unsafe {
                libc::pthread_cond_destroy(self.ptr);
            }
        }
        unsafe {
            ManuallyDrop::drop(&mut self.shm);
        }
    }
}

impl ShmCondition<MutableShmMap> {
    pub fn init_from_shm(shm: MutableShmMap, attr: ConditionAttr) -> nix::Result<Self> {
        let ptr = shm.start_ptr() as *mut pthread_cond_t;
        unsafe {
            let result = libc::pthread_cond_init(ptr, attr.ptr);
            if 0 == result {
                debug!("created cond at {:?}", shm.start_ptr());
                Ok(ShmCondition {
                    should_destroy: true,
                    shm: ManuallyDrop::new(shm),
                    ptr: ptr,
                })
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }

    pub fn notify_all(&mut self) -> nix::Result<()> {
        unsafe {
            let result = libc::pthread_cond_broadcast(self.ptr);
            if 0 == result {
                debug!("notified cond at {:?}", self.ptr);
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        }
    }
}

impl ShmCondition<ShmMap> {
    pub fn wait<T>(&mut self, mutex: &mut ShmMutex<T>) -> nix::Result<()> {
        let wait_result = mutex.lock().and_then(|_| unsafe {
            debug!("waiting on condition at {:?}", self.ptr);
            let wait_end = timespec {
                tv_sec: (SystemTime::now() + Duration::from_secs(30))
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                tv_nsec: 0,
            };
            let result = libc::pthread_cond_timedwait(self.ptr, mutex.ptr, &wait_end);
            if 0 == result {
                debug!("woke on condition at {:?}", self.ptr);
                Ok(())
            } else {
                Err(nix::errno::Errno::from_i32(result))
            }
        });
        let unlock_result = mutex.unlock();

        wait_result.and(unlock_result)
    }

    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut libc::pthread_cond_t;
        debug!("initialized cond at {:?}", ptr);
        ShmCondition {
            should_destroy: false,
            shm: ManuallyDrop::new(shm),
            ptr: ptr,
        }
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

    use super::{MutexAttr, ShmMutex};

    #[test]
    fn init_mutex_produces_a_valid_mutex_from_shared_memory() {
        let definition =
            ShmDefinition::new("sync".to_string(), std::mem::size_of::<pthread_mutex_t>());
        let shm = MutableShmMap::create(definition).unwrap();
        let attr = MutexAttr::init().unwrap();
        attr.set_pshared().unwrap();
        let mut mutex = ShmMutex::init_from_shm(shm, attr).unwrap();

        mutex.lock().unwrap();
        mutex.unlock().unwrap();
    }
}
