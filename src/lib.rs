use std::{time::SystemTime};

use futures::{Stream, StreamExt};

mod platform;

pub enum AppEvent {
    ToggleActivation,
    EnterPress(SystemTime)
}

pub struct AppEventStream(platform::AppEventStreamImpl);



// polyfill for unstable function
trait ResultInspect<F>  {
    fn inspect_err(self, op: F) -> Self;
}

impl<F, T, E, O> ResultInspect<F> for Result<T,E>
where F:FnOnce(&E) -> O {
    fn inspect_err(self, op: F) -> Self {
        if let Err(e) = &self {
            op(e);
        }
        return self
    }
}




impl AppEventStreamImpl for AppEventStream {
    fn new() -> Self {
        Self(platform::AppEventStreamImpl::new())
    }
}

impl Stream for AppEventStream {
    type Item = Result<AppEvent, anyhow::Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}
pub trait AppEventStreamImpl : Stream<Item=Result<AppEvent, anyhow::Error>> {
    fn new() -> Self;
}

