use vr_logger::{debug, error, info, init_global_logger, trace, warn};

fn main() {
    init_global_logger("log.txt".into(), "%r");
    trace!("hi");
    debug!("hi");
    info!("hi");
    warn!("hi");
    error!("hi");
}
