use std::thread::available_parallelism;

use vr_threading::init_global_threadpool;

fn main() {
    init_global_threadpool(available_parallelism().unwrap())
}
