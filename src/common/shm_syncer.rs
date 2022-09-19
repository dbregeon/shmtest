use std::sync::Mutex;

use super::{
    pthread::{self, ShmCondition},
    shm::{MutableShmMap, ShmMap},
    ShmDefinition,
};

pub struct ShmSync<T> {
    mutex: Mutex<u8>,
    condition: ShmCondition<T>,
}

impl ShmSync<MutableShmMap> {
    pub fn create(name: String) -> nix::Result<Self> {
        let condvar_definition = ShmDefinition::new(
            format!("{}_condvar", name),
            std::mem::size_of::<libc::pthread_cond_t>(),
        );
        let condvar_shm = MutableShmMap::create(condvar_definition).unwrap();
        let mutex = Mutex::new(0);
        let condition = pthread::ShmCondition::init_in_shm(condvar_shm);

        Ok(ShmSync { mutex, condition })
    }

    pub fn notify_all(&mut self) {
        self.condition.notify_all();
    }
}

impl ShmSync<ShmMap> {
    pub fn load(name: String) -> nix::Result<Self> {
        let condvar_definition = ShmDefinition::new(
            format!("{}_condvar", name),
            std::mem::size_of::<libc::pthread_cond_t>(),
        );
        let condvar_shm = ShmMap::open(condvar_definition).unwrap();
        let mutex = Mutex::new(0);
        let condition = pthread::ShmCondition::from_raw_pointer(condvar_shm);

        Ok(ShmSync { mutex, condition })
    }

    pub fn wait(&mut self) {
        self.condition.wait(&mut self.mutex);
    }
}
