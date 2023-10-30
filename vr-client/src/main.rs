use vr_logger::{debug, error, info, init_global_logger, trace, warn};

use std::thread::available_parallelism;

use vr_threading::init_global_threadpool;

fn main() {
    init_global_logger("log.txt".into(), "%r");
    init_global_threadpool(available_parallelism().unwrap())
    trace!("hi");
    debug!("hi");
    info!("hi");
    warn!("hi");
    error!("hi");
}
