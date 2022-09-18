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
        unsafe {
            let result = (*self.ptr).wait(mutex.lock()).unwrap();
            debug!("woke on condition at {:?} {:?}", self.ptr, result);
        }
    }

    pub fn from_raw_pointer(shm: ShmMap) -> Self {
        let ptr = shm.start_ptr() as *mut Condvar;
        debug!("initialized cond at {:?}", ptr);
        ShmCondition { shm: shm, ptr: ptr }
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

    use super::ShmMutex;

    #[test]
    fn init_mutex_produces_a_valid_mutex_from_shared_memory() {
        let owner = std::thread::spawn(|| {
            let mutex_definition =
                ShmDefinition::new("mutex".to_string(), std::mem::size_of::<Mutex<u8>>());
            let mutex_shm = MutableShmMap::create(mutex_definition).unwrap();
            let mut mutex = ShmMutex::init_in_shm(mutex_shm);
            let condition_definition =
                ShmDefinition::new("condition".to_string(), std::mem::size_of::<Condvar>());
            let condition_shm = MutableShmMap::create(condition_definition).unwrap();
            let mut condition = ShmCondition::init_in_shm(condition_shm);

            for _i in 0..10 {
                std::thread::sleep(Duration::from_secs(2));
                let mut guard = mutex.lock();
                *guard += 1;

                condition.notify_all();
            }
        });
        let client = std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(2));

            let mutex_definition =
                ShmDefinition::new("mutex".to_string(), std::mem::size_of::<Mutex<u8>>());
            let mutex_shm = ShmMap::open(mutex_definition).unwrap();
            let mut mutex = ShmMutex::from_raw_pointer(mutex_shm);
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
