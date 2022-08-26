use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Map<Fu, F> {
        #[pin]
        pub(crate) fu: Fu,
        pub(crate) f: Option<F>,
    }
}

impl<Fu, F, U> Future for Map<Fu, F>
where
    Fu: Future,
    F: FnOnce(Fu::Output) -> U,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Poll::Ready(r) = this.fu.poll(cx) {
            let f = this.f.take().expect("Map function must not be None");

            Poll::Ready(f(r))
        } else {
            Poll::Pending
        }
    }
}

pub trait MapFutureExt: Future {
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U,
        Self: Sized,
    {
        Map {
            fu: self,
            f: Some(f),
        }
    }
}

impl<F: Future + ?Sized> MapFutureExt for F {}
