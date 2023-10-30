use vr_logger::{debug, error, info, init_global_logger, trace, warn};


use std::{thread::sleep, time::Duration};

use vr_threading::{global_exec, global_init };

fn main() {
    init_global_logger("log.txt".into(), "%r");
    global_init();
    trace!("hi");
    debug!("hi");
    info!("hi");
    warn!("hi");
    error!("hi");

    global_exec(move || {
        global_exec(move || {
            sleep(Duration::from_secs(4));
        });
        sleep(Duration::from_secs(3));
    });
}
