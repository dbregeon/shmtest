use super::{
    pthread::{self, ShmCondition, ShmMutex},
    shm::{MutableShmMap, ShmMap},
    ShmDefinition,
};

pub struct ShmSync<T> {
    mutex: ShmMutex<T>,
    condition: ShmCondition<T>,
}

impl ShmSync<MutableShmMap> {
    pub fn create(name: String) -> nix::Result<Self> {
        let mutex_definition = ShmDefinition::new(
            format!("{}_mutex", name),
            std::mem::size_of::<libc::pthread_mutex_t>(),
        );
        let condvar_definition = ShmDefinition::new(
            format!("{}_condvar", name),
            std::mem::size_of::<libc::pthread_cond_t>(),
        );
        let mutex_shm = MutableShmMap::create(mutex_definition).unwrap();
        let condvar_shm = MutableShmMap::create(condvar_definition).unwrap();
        let mutex_attr = pthread::MutexAttr::init().unwrap();
        mutex_attr.set_pshared().unwrap();
        let mutex = pthread::ShmMutex::init_from_shm(mutex_shm, mutex_attr).unwrap();
        let cond_attr = pthread::ConditionAttr::init().unwrap();
        cond_attr.set_pshared().unwrap();
        let condition = pthread::ShmCondition::init_from_shm(condvar_shm, cond_attr).unwrap();

        Ok(ShmSync { mutex, condition })
    }

    pub fn notify_all(&mut self) -> nix::Result<()> {
        self.condition.notify_all()
    }
}

impl ShmSync<ShmMap> {
    pub fn load(name: String) -> nix::Result<Self> {
        let mutex_definition = ShmDefinition::new(
            format!("{}_mutex", name),
            std::mem::size_of::<libc::pthread_mutex_t>(),
        );
        let condvar_definition = ShmDefinition::new(
            format!("{}_condvar", name),
            std::mem::size_of::<libc::pthread_cond_t>(),
        );
        let mutex_shm = ShmMap::open(mutex_definition).unwrap();
        let condvar_shm = ShmMap::open(condvar_definition).unwrap();
        let mutex = pthread::ShmMutex::from_raw_pointer(mutex_shm);
        let condition = pthread::ShmCondition::from_raw_pointer(condvar_shm);

        Ok(ShmSync { mutex, condition })
    }

    pub fn wait(&mut self) -> nix::Result<()> {
        self.condition.wait(&mut self.mutex)
    }
}
