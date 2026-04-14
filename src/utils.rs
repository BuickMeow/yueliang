use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

struct DummyWaker;
impl Wake for DummyWaker {
    fn wake(self: Arc<Self>) {}
}

pub fn simple_block_on<F: Future>(mut future: F) -> F::Output {
    let waker = Waker::from(Arc::new(DummyWaker));
    let mut context = Context::from_waker(&waker);
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(val) => return val,
            Poll::Pending => thread::sleep(Duration::from_millis(10)),
        }
    }
}
