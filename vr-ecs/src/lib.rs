use std::sync::Arc;

enum AppState {}

struct App<S> {
    state: AppState,
    systems: Arc<[S]>,
}

impl<S: Fn()> App<S> {}

trait Entity {}
trait Component {}
