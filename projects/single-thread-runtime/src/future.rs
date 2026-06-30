use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Join<F, G>
where
    F: Future,
    G: Future,
{
    future1: Option<F>,
    future2: Option<G>,
    result1: Option<F::Output>,
    result2: Option<G::Output>,
}

impl<F, G> Join<F, G>
where
    F: Future + Unpin,
    G: Future + Unpin,
{
    pub fn new(future1: F, future2: G) -> Self {
        Self {
            future1: Some(future1),
            future2: Some(future2),
            result1: None,
            result2: None,
        }
    }
}

impl<F, G> Future for Join<F, G>
where
    F: Future + Unpin,
    G: Future + Unpin,
{
    type Output = (F::Output, G::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if let Some(future) = this.future1.as_mut() {
            if let Poll::Ready(result) = Pin::new(future).poll(cx) {
                this.result1 = Some(result);
                this.future1 = None;
            }
        }

        if let Some(future) = this.future2.as_mut() {
            if let Poll::Ready(result) = Pin::new(future).poll(cx) {
                this.result2 = Some(result);
                this.future2 = None;
            }
        }

        if this.result1.is_some() && this.result2.is_some() {
            let r1 = this.result1.take().unwrap();
            let r2 = this.result2.take().unwrap();
            Poll::Ready((r1, r2))
        } else {
            Poll::Pending
        }
    }
}