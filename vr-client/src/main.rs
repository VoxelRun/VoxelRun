use vr_logger::{debug, error, info, init_global_logger, trace, warn};

use std::thread::available_parallelism;

use vr_threading::init_global_threadpool;
use std::{thread::sleep, time::Duration};

use vr_threading::{global_exec, global_init};

fn main() {
    init_global_logger("log.txt".into(), "%r");
    init_global_threadpool(available_parallelism().unwrap())
    trace!("hi");
    debug!("hi");
    info!("hi");
    warn!("hi");
    error!("hi");

    THREADPOOL
        .lock()
        .unwrap()
        .execute(|| sleep(Duration::from_secs(3)));
    THREADPOOL.lock().unwrap().shutdown();
    global_init();
    global_exec(|| sleep(Duration::from_secs(3)))
}
