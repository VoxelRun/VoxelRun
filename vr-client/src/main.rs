use vr_logger::{debug, error, info, init_global_logger, trace, warn, DefaultLogger};

use std::{thread::sleep, time::Duration};

use vr_threading::{global_exec, global_init, global_promise};

fn main() {
    let logger = init_global_logger::<DefaultLogger>("log.txt".into(), None);
    let a = global_init();
    trace!("hi");
    debug!("hi");
    info!("hi");
    warn!("hi");
    error!("hi");

    global_exec(move || {
        global_exec(move || {
            sleep(Duration::from_secs(4));
            println!("hi");
        });
        sleep(Duration::from_secs(4));
        global_exec(move || {
            sleep(Duration::from_secs(4));
            println!("hi");
        });
        println!("hi2");
    });
    panic!("asdf");

    println!("{}", global_promise(|| 3).get());
    global_promise(|| {
        println!("a");
    })
    .get();
    drop(a);
    drop(logger);
}
