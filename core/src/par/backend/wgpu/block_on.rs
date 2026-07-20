//! Minimal thread-parking future executor for the wgpu backend.
//!
//! The GPU backend needs to resolve a handful of short-lived futures
//! synchronously (adapter/device requests, error-scope pops). A full executor
//! is unnecessary: poll the future on the current thread and park between
//! wakes. Drop-in replacement for `pollster::block_on` — `gpu-wgpu` implies
//! `std`, so `thread::park` is available.

use alloc::sync::Arc;
use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use std::thread::{self, Thread};

struct ThreadWaker(Thread);

impl std::task::Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

/// Run `future` to completion on the current thread, parking between polls.
///
/// Spurious unparks are harmless: the loop re-polls and parks again if the
/// future is still pending.
pub(super) fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => thread::park(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_ready_future() {
        assert_eq!(block_on(async { 41 + 1 }), 42);
    }

    #[test]
    fn resolves_future_woken_from_another_thread() {
        struct Handoff(std::sync::Mutex<(Option<i32>, Option<Waker>)>);

        impl Future for &Handoff {
            type Output = i32;

            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
                let mut state = self.0.lock().expect("handoff lock poisoned");
                if let Some(v) = state.0.take() {
                    Poll::Ready(v)
                } else {
                    state.1 = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }

        let handoff = Arc::new(Handoff(std::sync::Mutex::new((None, None))));
        let producer = Arc::clone(&handoff);
        let t = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let mut state = producer.0.lock().expect("handoff lock poisoned");
            state.0 = Some(7);
            if let Some(waker) = state.1.take() {
                waker.wake();
            }
        });
        assert_eq!(block_on(&*handoff), 7);
        t.join().expect("producer thread panicked");
    }
}
