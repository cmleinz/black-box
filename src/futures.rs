use std::{future::Future, task::Poll};

use pin_project_lite::pin_project;

/// Race two futures against one another, favoring the first future over the second
pub fn race_biased<F1, F2>(first: F1, second: F2) -> RaceBiased<F1, F2> {
    RaceBiased { first, second }
}

pin_project! {
    pub struct RaceBiased<F1, F2> {
        #[pin]
        first: F1,
        #[pin]
        second: F2,
    }
}

impl<F1, F2, T> Future for RaceBiased<F1, F2>
where
    F1: Future<Output = T>,
    F2: Future<Output = T>,
{
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(val) = this.first.poll(cx) {
            Poll::Ready(val)
        } else if let Poll::Ready(val) = this.second.poll(cx) {
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}
