use std::sync::{atomic::Ordering, Condvar, Mutex, MutexGuard};

use linux_futex::{Futex, Shared, WaitError};
use log::debug;

use super::shm::{MutableShmMap, ShmMap};

pub struct ShmMutex<T> {
    _shm: T,
    ptr: *mut Mutex<u8>,
}

impl ShmMutex<MutableShmMap> {
    pub fn init_in_shm(shm: MutableShmMap) -> Self {
        unsafe {
            let ptr = shm.start_ptr() as *mut Mutex<u8>;
            debug!("created mutex at {:?}", *shm.start_ptr());
            ShmMutex::<MutableShmMap> {
                _shm: shm,
                ptr: ptr,
            }
        }
    }
}

impl<T> ShmMutex<T> {
    pub fn lock(&mut self) -> MutexGuard<u8> {
        unsafe {
            debug!("locking mutex at {:?}", *self.ptr);
            let guard = (*self.ptr).lock().unwrap();
            debug!("locked mutex at {:?}", *self.ptr);
            guard
        }
    }
}

impl ShmMutex<ShmMap> {
    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Mutex<u8>;
        unsafe { debug!("initialized mutex at {:?}", *ptr) };
        ShmMutex {
            _shm: shm,
            ptr: ptr,
        }
    }
}

pub struct ShmCondition<T> {
    _shm: T,
    ptr: *mut Futex<Shared>,
}

impl ShmCondition<MutableShmMap> {
    pub fn init_in_shm(shm: MutableShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Futex<Shared>;
        unsafe {
            (*ptr).value.store(1, Ordering::Relaxed);
            debug!("created cond at {:?}", *ptr);
            ShmCondition {
                _shm: shm,
                ptr: ptr,
            }
        }
    }

    pub fn notify_all(&mut self) {
        unsafe {
            (*self.ptr).value.fetch_add(1, Ordering::Release);
            (*self.ptr).wake(libc::INT_MAX);
            debug!("notified cond at {:?}", *self.ptr);
        }
    }
}

impl<T> ShmCondition<T> {
    pub fn wait(&mut self, mutex: &mut Mutex<u8>) {
        unsafe {
            debug!("Waiting on condition at {:?}", *self.ptr);
            let expected_value = {
                let _guard = mutex.lock();
                (*self.ptr).value.load(Ordering::Acquire)
            };
            loop {
                let result = (*self.ptr).wait(expected_value);
                match result {
                    Err(WaitError::Interrupted) => continue,
                    Err(WaitError::WrongValue) => return,
                    _ => {
                        debug!("woke on condition at {:?} {:?}", *self.ptr, result);
                        return;
                    }
                }
            }
        }
    }
}

impl ShmCondition<ShmMap> {
    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Futex<Shared>;
        unsafe { debug!("initialized cond at {:?}", *ptr) };
        ShmCondition {
            _shm: shm,
            ptr: ptr,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Condvar, Mutex},
        time::Duration,
    };

    use crate::common::{
        pthread::ShmCondition,
        shm::{MutableShmMap, ShmMap},
        ShmDefinition,
    };

    #[test_log::test]
    fn init_mutex_produces_a_valid_mutex_from_shared_memory() {
        let owner = std::thread::spawn(|| {
            let condition_definition =
                ShmDefinition::new("condition".to_string(), std::mem::size_of::<Condvar>());
            let condition_shm1 = MutableShmMap::create(condition_definition).unwrap();
            let mut condition1 = ShmCondition::init_in_shm(condition_shm1);

            for _i in 0..5 {
                std::thread::sleep(Duration::from_secs(1));
                condition1.notify_all();
            }
        });
        let client = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(2));
            let mut mutex = Mutex::new(0);
            let condvar_definition =
                ShmDefinition::new("condition".to_string(), std::mem::size_of::<Condvar>());
            let condvar_shm = ShmMap::open(condvar_definition).unwrap();
            let mut condition = ShmCondition::from_raw_pointer(condvar_shm);

            condition.wait(&mut mutex);
            true
        });
        assert!(client.join().unwrap());
        owner.join().unwrap();
    }
}
