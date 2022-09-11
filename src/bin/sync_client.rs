use log::info;
use shmtest::common::{shm::ShmMap, shm_syncer::ShmSync};

fn main() {
    env_logger::init();

    let mut syncer = ShmSync::<ShmMap>::load("test".to_string()).unwrap();

    info!("start wait");
    syncer.wait().unwrap();
    info!("end wait");
}
