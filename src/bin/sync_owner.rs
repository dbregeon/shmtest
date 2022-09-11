use log::info;
use shmtest::common::{shm::MutableShmMap, shm_syncer::ShmSync};

fn main() {
    env_logger::init();

    let mut syncer = ShmSync::<MutableShmMap>::create("test".to_string()).unwrap();
    syncer.notify_all().unwrap();

    info!("Sleeping");
    std::thread::sleep(std::time::Duration::from_secs(30));

    for i in 0..30 {
        syncer.notify_all().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    info!("Finished");
}
