
use std::{task::Poll};

use evdev::{EventStream, InputEventKind, Key};
use futures::{Stream, StreamExt};
use tokio_stream::StreamMap;

use crate::AppEventStreamImpl;
use crate::AppEvent;
use crate::ResultInspect;

pub(crate) struct AppEventStream {
    inner: StreamMap<usize, EventStream>
}

impl AppEventStreamImpl for AppEventStream {
    fn new() -> Self {
        Self {
            inner: evdev::enumerate()
                .filter(|(_,dev)| dev.supported_keys().is_some_and(|x| x.contains(Key::KEY_H) && x.contains(Key::KEY_ENTER)))
                .filter_map(|(_,dev)| {
                    // because Device::into_event_stream will drop the device on Error, 
                    // we have no way to get the name from the device without cloning it out of the device beforehand.
                    // I have created https://github.com/emberian/evdev/issues/144 which relies on https://github.com/tokio-rs/tokio/issues/6344
                    // but in the meantime this is the best solution
                    let name = dev.name().unwrap_or("[unnamed device]").to_owned();
                    // I had to implement inspect_err myself because it's not yet stable. compiler warns about using a name used by an unstable function
                    #[allow(unstable_name_collisions)]
                    dev.into_event_stream().inspect_err(|e|println!("Could not initialize device {name}, got error {e}")).ok()
                })
                .enumerate()
                .collect()
        }
    }
}

impl Stream for AppEventStream {
    type Item = Result<AppEvent, anyhow::Error>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {

        while let Poll::Ready(data) = self.inner.poll_next_unpin(cx) {
            match data {
                // End of Future
                None => return Poll::Ready(None),
                // H Key Press
                Some((_, Ok(event))) if (event.kind(), event.value()) == (InputEventKind::Key(Key::KEY_H), 0 ) => 
                    return Poll::Ready(Some(Ok(AppEvent::ToggleActivation)))
                ,
                // Enter Key Press
                Some((_, Ok(event))) if (event.kind(), event.value()) == (InputEventKind::Key(Key::KEY_ENTER), 0 ) => 
                    return Poll::Ready(Some(Ok(AppEvent::EnterPress(event.timestamp()))))
                ,
                // Error
                Some((_, Err(err))) => 
                    return Poll::Ready(Some(Err(err.into()))),
                // Something Else
                _ => {}
            }
        }
        std::task::Poll::Pending
    }
}